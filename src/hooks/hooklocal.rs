use crate::buttons;
use crate::config;
use crate::config::REMAP_SMALLVEC_LEN;
use enum_map::EnumMap;
use smallvec::SmallVec;
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{debug, info, instrument, warn};

/*
    This is the runtime storage for the Hook thread.
    I also dislike global variables. Unfortunately, the nature of Windows hook callbacks make it
    necessary.

    Everything that touches this is in the hooks module:
        hooks/mod.rs:   initializes HOOKLOCAL on startup,
                        updates HOOKLOCAL.settings on receipt of an Update message
        hooks/input.rs: reads and updates HOOKLOCAL on every button press and release

    TODO: see about swapping the Mutex with something else
*/
pub static HOOKLOCAL: Mutex<Option<HookLocalData>> = Mutex::new(None);

// -------------------- HookLocalData --------------------
#[derive(Debug, Clone, Default)]
pub struct HookLocalData {
    pub config: config::Config,
    pub button_state: EnumMap<buttons::HoldButton, HoldButtonState>,
    pub active_profile: ActiveProfile,
    pub active_layers_default: SmallVec<[bool; REMAP_SMALLVEC_LEN]>,
    pub active_layers_profile: Vec<SmallVec<[bool; REMAP_SMALLVEC_LEN]>>, // Outer vec: over profiles. Inner vec: over layers.
}

impl HookLocalData {
    /// Create a new HookLocalData struct instance from an initial configuration.
    pub fn init_settings(config: config::Config) -> Self {
        let mut result = Self::default();
        result.update_config(config);
        result
    }

    pub fn update_config(&mut self, config: config::Config) {
        use smallvec::smallvec;

        self.config = config;
        self.active_layers_default = smallvec![false; self.config.default.layers.len()];
        self.active_layers_profile = self
            .config
            .profiles
            .iter()
            .map(|profile| smallvec![false; profile.layers.len()])
            .collect();
        self.update_active_profile();
    }

    /// Check which window is in the foreground and update the active profile accordingly.
    pub fn update_active_profile(&mut self) {
        let Some(ForegroundWindowInfo { title, process }) = get_foreground_window() else {
            warn!("failed to get foreground window; not updating the active profile");
            return;
        };
        let mut new_profile = ActiveProfile::Default;
        for (i, profile_condition) in self
            .config
            .profiles
            .iter()
            .enumerate()
            .filter(|(_, profile)| profile.enabled)
            .map(|(i, profile)| (i, &profile.condition))
        {
            match profile_condition {
                // Non-DE
                // Title: "Ori And The Blind Forest"
                // Process: "ori.exe"

                // Definitive Edition
                // Title: "Ori And The Blind Forest: Definitive Edition"
                // Process: "oriDE.exe"
                config::ProfileCondition::OriBF => {
                    if title == "Ori And The Blind Forest: Definitive Edition" {
                        new_profile = ActiveProfile::Other(i);
                    }
                }
                // Title: "OriAndTheWilloftheWisps"
                // Process: "oriwotw.exe"
                config::ProfileCondition::OriWotW => {
                    if title == "OriAndTheWilloftheWisps" {
                        new_profile = ActiveProfile::Other(i);
                    }
                }
                config::ProfileCondition::Other(condition_title) => {
                    if title == *condition_title {
                        new_profile = ActiveProfile::Other(i);
                    }
                }
            }
        }
        if self.active_profile != new_profile {
            match new_profile {
                ActiveProfile::Default => info!(?new_profile, "switching to default profile"),
                ActiveProfile::Other(profile_idx) => info!(
                    ?new_profile,
                    "switching to profile {}", &self.config.profiles[profile_idx].name
                ),
            }
        }
        self.active_profile = new_profile;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ForegroundWindowInfo {
    title: String,
    process: String,
}

#[instrument]
fn get_foreground_window() -> Option<ForegroundWindowInfo> {
    use windows::Win32::UI::WindowsAndMessaging as WM;
    debug!("getting foreground window");

    let hwnd = unsafe { WM::GetForegroundWindow() };
    if hwnd.is_invalid() {
        warn!("foreground window handle is invalid");
        return None;
    }

    let title = unsafe { get_window_title(hwnd) }?;
    debug!(?title, "got foreground title");
    let process = unsafe { get_process_name(hwnd) }?;
    debug!(?process, "got foreground process");

    Some(ForegroundWindowInfo { title, process })
}

// SAFETY: hwnd must be valid
#[instrument]
unsafe fn get_window_title(hwnd: windows::Win32::Foundation::HWND) -> Option<String> {
    use windows::Win32::UI::WindowsAndMessaging as WM;

    const CAP: usize = 512;

    let mut title = [0u16; CAP];
    let len = unsafe { WM::GetWindowTextW(hwnd, &mut title) };
    if len == 0 {
        warn!("window text is empty");
        return None;
    }
    let len = std::cmp::min(len as usize, CAP - 1);
    Some(String::from_utf16_lossy(&title[0..len]))
}

// SAFETY: hwnd must be valid
#[instrument]
unsafe fn get_process_name(hwnd: windows::Win32::Foundation::HWND) -> Option<String> {
    use windows::Win32::System::Threading as TH;
    use windows::Win32::UI::WindowsAndMessaging as WM;
    use windows::core::PWSTR;

    const CAP: usize = 512;

    let mut lpdwprocessid = 0u32;
    let lpdwprocessid_ptr: *mut u32 = &mut lpdwprocessid;
    let dwprocessid = unsafe { WM::GetWindowThreadProcessId(hwnd, Some(lpdwprocessid_ptr)) };
    if dwprocessid == 0 {
        warn!("thread id is invalid");
        return None;
    }

    let hprocess = unsafe {
        match TH::OpenProcess(TH::PROCESS_QUERY_LIMITED_INFORMATION, false, lpdwprocessid) {
            Ok(hprocess) => hprocess,
            Err(e) => {
                warn!("failed to open process: {}", e);
                return None;
            }
        }
    };

    let mut title = [0u16; CAP];
    let mut len = (CAP - 1) as u32;
    match unsafe {
        TH::QueryFullProcessImageNameW(
            hprocess,
            TH::PROCESS_NAME_WIN32,
            PWSTR(&mut title as *mut u16),
            &mut len,
        )
    } {
        Ok(()) => (),
        Err(e) => {
            warn!("error querying process image name: {e}");
            return None;
        }
    };
    let len = std::cmp::min(len as usize, CAP - 1);
    let title = String::from_utf16_lossy(&title[0..len]);
    let title = PathBuf::from(title);
    let Some(title) = title.file_name() else {
        warn!("could not get as filename");
        return None;
    };
    Some(String::from(title.to_string_lossy()))
}

// -------------------- ActiveProfile --------------------
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActiveProfile {
    Default,
    Other(usize),
}

impl Default for ActiveProfile {
    fn default() -> Self {
        Self::Default
    }
}

// -------------------- HoldButtonState --------------------
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HoldButtonState {
    NotHeld,
    HeldNoRemap,
    HeldWithRemap(config::Output),
}

impl Default for HoldButtonState {
    fn default() -> Self {
        Self::NotHeld
    }
}
