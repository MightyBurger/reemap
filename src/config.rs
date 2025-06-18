use crate::buttons::{Button, HoldButton};
use enum_map::EnumMap;

/*
    There is one config. The config contains multiple profiles. Each profile contains multiple
    layers. Each layer contains multiple remaps.

    Config -> Profiles -> Layers -> Remaps

    A profile corresponds to a window. For example, the user may have certain settings apply when
    Ori and the Blind Forest is the foreground window.  The Default profile is active when no other
    profile is active.

*/

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BaseRemapPolicy {
    NoRemap,
    Remap(Vec<Button>),
}

impl Default for BaseRemapPolicy {
    fn default() -> Self {
        Self::NoRemap
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RemapPolicy {
    Defer,
    NoRemap,
    Remap(Vec<Button>),
}

impl Default for RemapPolicy {
    fn default() -> Self {
        Self::Defer
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BaseLayer {
    pub policy: EnumMap<Button, BaseRemapPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayerType {
    Modifier,
    Toggle,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Layer {
    pub active: bool,
    pub layer_type: LayerType,
    pub condition: Vec<HoldButton>,
    pub policy: EnumMap<Button, RemapPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProfileCondition {
    OriBF,
    OriWotW,
    Other(String),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Profile {
    pub base: BaseLayer,
    pub layers: Vec<Layer>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Config {
    pub default: Profile,
    pub profiles: Vec<Profile>,
    pub profile_conditions: Vec<ProfileCondition>,
    pub active_profile: Option<usize>,
}

impl Config {
    pub fn get_active_profile_mut(&mut self) -> &mut Profile {
        if let Some(active_profile_idx) = self.active_profile {
            self.profiles
                .get_mut(active_profile_idx)
                .expect("active profile idx should always be valid")
        } else {
            &mut self.default
        }
    }
}
