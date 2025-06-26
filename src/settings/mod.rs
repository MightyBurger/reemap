use crate::buttons;
use crate::config;
use enum_map::EnumMap;
use serde::Deserialize;
use serde::Serialize;

// -------------------- Config (UI) --------------------

// Like config::Config, but instantiates ProfileUI instead of Profile and without active_profile
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConfigUI {
    pub default: DefaultProfileUI,
    pub profiles: Vec<ProfileUI>,
}

impl From<ConfigUI> for config::Config {
    fn from(value: ConfigUI) -> Self {
        Self {
            default: value.default.into(),
            profiles: value
                .profiles
                .iter()
                .cloned()
                .filter_map(|profiles_ui| profiles_ui.try_into().ok())
                .collect(),
            profile_conditions: value
                .profiles
                .into_iter()
                .map(|profiles_ui| profiles_ui.condition)
                .collect(),
            active_profile: None,
        }
    }
}

// -------------------- Profiles (UI) --------------------
// Like config::Profile, but  uses Vec<LayerUI> instead of Vec<config::Layer>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DefaultProfileUI {
    pub base: config::BaseLayer,
    pub layers: Vec<LayerUI>,
}

impl Default for DefaultProfileUI {
    fn default() -> Self {
        Self {
            base: config::BaseLayer::default(),
            layers: Vec::new(),
        }
    }
}

impl From<DefaultProfileUI> for config::DefaultProfile {
    fn from(value: DefaultProfileUI) -> Self {
        Self {
            base: value.base,
            layers: value
                .layers
                .into_iter()
                .filter_map(|layer_ui| layer_ui.try_into().ok())
                .collect(),
        }
    }
}

// Like config::Profile, but with extra fields:
//  enabled
//  name
// Also uses Vec<LayerUI> instead of Vec<config::Layer>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProfileUI {
    pub base: config::BaseLayer,
    pub layers: Vec<LayerUI>,
    pub condition: config::ProfileCondition,
    pub enabled: bool,
    pub name: String,
}

impl Default for ProfileUI {
    fn default() -> Self {
        Self {
            base: config::BaseLayer::default(),
            layers: Vec::new(),
            condition: config::ProfileCondition::OriBF,
            enabled: true,
            name: String::from("New Profile"),
        }
    }
}

impl TryFrom<ProfileUI> for config::Profile {
    type Error = ();
    fn try_from(value: ProfileUI) -> Result<Self, ()> {
        if !value.enabled {
            Err(())
        } else {
            Ok(Self {
                base: value.base,
                layers: value
                    .layers
                    .into_iter()
                    .filter_map(|layer_ui| layer_ui.try_into().ok())
                    .collect(),
                condition: value.condition,
            })
        }
    }
}

// -------------------- Layers (UI) --------------------
// Like config::Layer, but with extra fields:
//  enabled
//  name
// and without these fields:
//  active
// ("enabled" means the user clicked the checkbox for this layer. "active" means the layer is
// currently in effect; for example, the user is holding down the required buttons.)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LayerUI {
    pub layer_type: config::LayerType,
    pub condition: Vec<buttons::HoldButton>,
    pub policy: EnumMap<buttons::Button, config::RemapPolicy>,
    pub enabled: bool,
    pub name: String,
}

impl Default for LayerUI {
    fn default() -> Self {
        Self {
            enabled: true,
            layer_type: config::LayerType::Modifier,
            condition: Vec::new(),
            policy: EnumMap::default(),
            name: String::from("New Layer"),
        }
    }
}

impl TryFrom<LayerUI> for config::Layer {
    type Error = ();
    fn try_from(value: LayerUI) -> Result<Self, ()> {
        if !value.enabled {
            Err(())
        } else {
            Ok(Self {
                active: false,
                layer_type: value.layer_type,
                condition: value.condition,
                policy: value.policy,
            })
        }
    }
}
