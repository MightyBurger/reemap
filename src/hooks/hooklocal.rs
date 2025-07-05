use crate::buttons;
use crate::config;
use crate::config::REMAP_SMALLVEC_LEN;
use crate::hooks::query_foreground::ForegroundWindowInfo;
use crate::hooks::query_foreground::get_foreground_window;
use enum_map::EnumMap;
use smallvec::SmallVec;
use std::sync::Mutex;
use tracing::{info, warn};

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

    /// Change the remaps to the provided configuration
    pub fn update_config(&mut self, config: config::Config) {
        use smallvec::smallvec;

        // Set self.config, .active_profile, .active_layers_default, and .active_layers_profile.
        // Note: it is not necessary to set button_state.

        self.config = config;
        self.active_layers_default = smallvec![false; self.config.default.layers.len()];
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
                self.active_profile = ActiveProfile::Default;
            }
        }
    }

    /// Update the active profile using information about the current foreground window.
    pub fn update_from_foreground(&mut self, info: ForegroundWindowInfo) {
        let ForegroundWindowInfo { title, process } = info;
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
