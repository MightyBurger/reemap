use crate::buttons;
use crate::settings::Settings;
use enum_map::EnumMap;
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HoldButtonState {
    NotHeld,
    HeldNoRemap,
    HeldWithRemap(Vec<buttons::Button>),
}

impl Default for HoldButtonState {
    fn default() -> Self {
        Self::NotHeld
    }
}

#[derive(Debug, Clone, Default)]
pub struct HookLocalData {
    pub button_state: EnumMap<buttons::HoldButton, HoldButtonState>,
    pub settings: Settings,
}
