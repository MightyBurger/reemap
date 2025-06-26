use crate::buttons::{Button, HoldButton};
use enum_map::EnumMap;
use serde::{Deserialize, Serialize};

/*
    There is one config. The config contains multiple profiles. Each profile contains multiple
    layers. Each layer contains multiple remaps.

    Config -> Profiles -> Layers -> Remaps

    A profile corresponds to a window. For example, the user may have certain settings apply when
    Ori and the Blind Forest is the foreground window.  The Default profile is active when no other
    profile is active.

*/

// -------------------- Remaps --------------------

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BaseRemapPolicy {
    NoRemap,
    Remap(Vec<Button>),
}

impl Default for BaseRemapPolicy {
    fn default() -> Self {
        Self::NoRemap
    }
}

impl std::fmt::Display for BaseRemapPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoRemap => write!(f, "(do not remap)"),
            Self::Remap(maps) => {
                if maps.len() == 0 {
                    return write!(f, "(block input)");
                }
                let outstr: String = itertools::Itertools::intersperse(
                    maps.iter().map(|btn| format!("{btn}")),
                    String::from(", "),
                )
                .collect();
                write!(f, "{outstr}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RemapPolicy {
    Defer,
    NoRemap,
    Remap(Vec<Button>),
}

impl std::fmt::Display for RemapPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Defer => write!(f, "(defer to next layer)"),
            Self::NoRemap => write!(f, "(do not remap)"),
            Self::Remap(maps) => {
                if maps.len() == 0 {
                    return write!(f, "(block input)");
                }
                let outstr: String = itertools::Itertools::intersperse(
                    maps.iter().map(|btn| format!("{btn}")),
                    String::from(", "),
                )
                .collect();
                write!(f, "{outstr}")
            }
        }
    }
}

impl Default for RemapPolicy {
    fn default() -> Self {
        Self::Defer
    }
}

// -------------------- Layers --------------------

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BaseLayer {
    pub policy: EnumMap<Button, BaseRemapPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LayerType {
    Modifier,
    Toggle,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Layer {
    pub active: bool,
    pub layer_type: LayerType,
    pub condition: Vec<HoldButton>,
    pub policy: EnumMap<Button, RemapPolicy>,
}

// -------------------- Profiles --------------------

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ProfileCondition {
    OriBF,
    OriWotW,
    Other(String),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DefaultProfile {
    pub base: BaseLayer,
    pub layers: Vec<Layer>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Profile {
    pub base: BaseLayer,
    pub layers: Vec<Layer>,
    pub condition: ProfileCondition,
}

// -------------------- Config --------------------

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Config {
    pub default: DefaultProfile,
    pub profiles: Vec<Profile>,
    pub profile_conditions: Vec<ProfileCondition>,
    pub active_profile: Option<usize>,
}

impl Config {
    pub fn get_active_base_layer_mut(&mut self) -> &mut BaseLayer {
        if let Some(active_profile_idx) = self.active_profile {
            &mut self.profiles[active_profile_idx].base
        } else {
            &mut self.default.base
        }
    }
    pub fn get_active_base_layer(&self) -> &BaseLayer {
        if let Some(active_profile_idx) = self.active_profile {
            &self.profiles[active_profile_idx].base
        } else {
            &self.default.base
        }
    }
    pub fn get_active_layers_mut(&mut self) -> &mut Vec<Layer> {
        if let Some(active_profile_idx) = self.active_profile {
            &mut self.profiles[active_profile_idx].layers
        } else {
            &mut self.default.layers
        }
    }
    pub fn get_active_layers(&self) -> &Vec<Layer> {
        if let Some(active_profile_idx) = self.active_profile {
            &self.profiles[active_profile_idx].layers
        } else {
            &self.default.layers
        }
    }
}
