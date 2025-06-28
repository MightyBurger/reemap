use crate::buttons;
use crate::config;
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
    pub active_layers: SmallVec<[bool; 8]>,
}

impl HookLocalData {
    pub fn init_settings(config: config::Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
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
