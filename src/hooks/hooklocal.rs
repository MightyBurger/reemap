use crate::buttons;
use crate::config;
use crate::config::REMAP_SMALLVEC_LEN;
use enum_map::EnumMap;
use smallvec::SmallVec;
use std::sync::Mutex;

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
        let foreground = get_foreground_window();
        self.active_profile = ActiveProfile::Default;
        for (i, profile_condition) in self
            .config
            .profiles
            .iter()
            .enumerate()
            .filter(|(_, profile)| profile.enabled)
            .map(|(i, profile)| (i, &profile.condition))
        {
            match profile_condition {
                config::ProfileCondition::OriBF => {
                    if foreground == "Ori And The Blind Forest: Definitive Edition" {
                        self.active_profile = ActiveProfile::Other(i);
                    }
                }
                config::ProfileCondition::OriWotW => {
                    if foreground == "OriAndTheWilloftheWisps" {
                        self.active_profile = ActiveProfile::Other(i);
                    }
                }
                config::ProfileCondition::Other(title) => {
                    if foreground == *title {
                        self.active_profile = ActiveProfile::Other(i);
                    }
                }
            }
        }
    }
}

fn get_foreground_window() -> String {
    use windows::Win32::UI::WindowsAndMessaging as WM;
    const CAP: usize = 512;
    let hwnd = unsafe { WM::GetForegroundWindow() };
    let mut title_u16 = [0u16; CAP];
    let len = unsafe { WM::GetWindowTextW(hwnd, &mut title_u16) };
    let len = std::cmp::max(len as usize, CAP - 1);
    String::from_utf16_lossy(&title_u16[0..len])
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
