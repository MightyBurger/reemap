use crate::buttons;
use crate::config;
use crate::config::REMAP_SMALLVEC_LEN;
use crate::gui;
use crate::gui::ReemapGuiEvent;
use crate::query_windows::WindowInfo;
use crate::query_windows::get_foreground_window;
use enum_map::EnumMap;
use smallvec::SmallVec;
use std::sync::Mutex;
use tracing::{info, warn};
use windows::Win32::Foundation;

/*
    This is the runtime storage for the Hook thread.
    I also dislike global variables. Unfortunately, the nature of Windows hook callbacks make it
    necessary.

    Everything that touches this variable is in the hooks module:
        hooks/input.rs (the main user of this data):
            -   acquires the mutex and changes HOOKLOCAL on every button press and release
        hooks/mod.rs:
            -   initializes HOOKLOCAL on startup
            -   acquires the mutex and calls .update_config() on recepit of an Update message
            -   acquires the mutex and calls .update_new_foreground() on receipt of a Check
                Foreground Window message
*/
pub static HOOKLOCAL: Mutex<Option<HookLocalData>> = Mutex::new(None);

// -------------------- HookLocalData --------------------
#[derive(Debug, Clone)]
pub struct HookLocalData {
    pub ui_proxy: winit::event_loop::EventLoopProxy<gui::ReemapGuiEvent>,
    pub ui_observing_inputs: bool,
    pub config: config::Config,
    pub button_state: EnumMap<buttons::HoldButton, HoldButtonState>,
    pub active_profile: Option<usize>,
    pub active_layers_profile: Vec<SmallVec<[bool; REMAP_SMALLVEC_LEN]>>, // Outer vec: over profiles. Inner vec: over layers.
    pub last_clip: Option<Foundation::RECT>,
}

impl HookLocalData {
    /// Create a new HookLocalData struct instance from an initial configuration.
    pub fn init_settings(
        config: config::Config,
        ui_proxy: winit::event_loop::EventLoopProxy<ReemapGuiEvent>,
    ) -> Self {
        let mut result = Self {
            ui_proxy,
            ui_observing_inputs: false,
            config: Default::default(),
            button_state: Default::default(),
            active_profile: Default::default(),
            active_layers_profile: Default::default(),
            last_clip: Default::default(),
        };
        result.update_config(config);
        result
    }

    /// Change the remaps to the provided configuration
    pub fn update_config(&mut self, config: config::Config) {
        use smallvec::smallvec;

        // Set self.config, .active_profile, .active_layers
        // Note: it is not necessary to set button_state.

        self.config = config;
        self.active_layers_profile = self
            .config
            .profiles
            .iter()
            .map(|profile| smallvec![false; profile.layers.len()])
            .collect();

        match get_foreground_window() {
            Ok(info) => {
                self.update_from_foreground(info);
            }
            Err(e) => {
                warn!(
                    ?e,
                    "failed to get foreground window; assuming default profile"
                );
                self.active_profile = None;
            }
        }
    }

    /// Update the active profile using information about the current foreground window.
    pub fn update_from_foreground(&mut self, info: WindowInfo) {
        use windows::Win32::UI::Input::KeyboardAndMouse as KBM;
        use windows::Win32::UI::WindowsAndMessaging as WM;

        let WindowInfo {
            title,
            process,
            rect,
        } = info;

        let mut new_profile = None;
        for (i, profile_condition) in self
            .config
            .profiles
            .iter()
            .enumerate()
            .filter(|(_, profile)| profile.enabled)
            .map(|(i, profile)| (i, &profile.condition))
        {
            let profile_matches = match profile_condition {
                config::ProfileCondition::Always => true,
                config::ProfileCondition::TitleAndProcess {
                    title: condition_title,
                    process: condition_process,
                } if title == *condition_title && process == *condition_process => true,

                config::ProfileCondition::Title {
                    title: condition_title,
                } if title == *condition_title => true,

                config::ProfileCondition::Process {
                    process: condition_process,
                } if process == *condition_process => true,
                _ => false,
            };
            if profile_matches {
                new_profile = Some(i);
                break;
            }
        }
        if self.active_profile != new_profile {
            // Inform the UI thread the profile changed.
            // There's a possibility the UI thread just barely stopped, so this may fail.
            // That's OK, so we intentionally ignore any errors.
            let ui_send_result = self
                .ui_proxy
                .send_event(gui::ReemapGuiEvent::ChangedProfile(
                    new_profile.map(|profile_idx| self.config.profiles[profile_idx].name.clone()),
                ));
            if ui_send_result.is_err() {
                warn!("failed to send message to UI thread");
            }
            match new_profile {
                None => info!(?new_profile, "no profile enabled"),
                Some(profile_idx) => info!(
                    ?new_profile,
                    "switching to profile {}", &self.config.profiles[profile_idx].name
                ),
            }
        }
        self.active_profile = new_profile;

        // Finally, update the cursor clip.
        // We should clip only if:
        //  -   scroll lock is not enabled, and
        //  -   the profile wants it, and
        //  -   we successfully got the window bounds
        let profile_wants_to_clip_cursor = match self.active_profile {
            None => false,
            Some(idx) => self.config.profiles[idx].clip_cursor,
        };

        let scroll_lock = unsafe { KBM::GetKeyState(KBM::VK_SCROLL.0.into()) & 1 > 0 };

        let will_clip_to = if !scroll_lock
            && profile_wants_to_clip_cursor
            && let Some(rect) = rect
        {
            Some(rect)
        } else {
            None
        };

        // Avoid calling ClipCursor excessively.
        // This is more important for not calling ClipCursor(None) all the time. We don't want to
        // stop other programs from clipping the cursor as they please. We just want to release
        // our cursor clip when we're done.
        if will_clip_to == self.last_clip {
            return;
        }
        self.last_clip = will_clip_to;

        info!(?will_clip_to, "clipping");

        // Clip the cursor!
        // as_ref() is preferable to this, except Option<&T> doesn't get coerced to Option<*const T>
        let clip_result = unsafe {
            match will_clip_to {
                None => WM::ClipCursor(None),
                Some(rect) => WM::ClipCursor(Some(&rect)),
            }
        };

        match clip_result {
            Ok(()) => (),
            Err(e) => warn!(?e, "failed to clip cursor"),
        }
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
