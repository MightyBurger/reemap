// use std::io::Read;

use crate::buttons;
use crate::settings;
use enum_map::EnumMap;
use serde::Deserialize;
use serde::Serialize;
// use thiserror::Error;

// -------------------- VersionedConfig --------------------
// Preparing for the future when the Config struct may change.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum VersionedConfig {
    V1(ConfigUI),
}

impl Default for VersionedConfig {
    fn default() -> Self {
        Self::V1(ConfigUI::default())
    }
}

// Even with future versions, this From<> will be from the latest Config to VersionedConfig.
impl From<ConfigUI> for VersionedConfig {
    fn from(value: ConfigUI) -> Self {
        Self::V1(value)
    }
}

// In future versions, this From<> will need a little more logic to do migration to the newest
// config version. Right now, as there's only one version of the config, there are no migrations.
impl From<VersionedConfig> for ConfigUI {
    fn from(value: VersionedConfig) -> Self {
        match value {
            VersionedConfig::V1(config_ui) => config_ui,
        }
    }
}

// -------------------- Config (UI) --------------------

// Like config::Settings, but instantiates ProfileUI instead of Profile and without active_profile
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConfigUI {
    pub default: DefaultProfileUI,
    pub profiles: Vec<ProfileUI>,
}

impl From<ConfigUI> for settings::Settings {
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

// impl ConfigUI {
//     fn save(&self, file: &mut std::fs::File) -> ConfigUIResult<()> {
//         let config_str = ron::ser::to_string_pretty(
//             &VersionedConfig::from(self.clone()),
//             ron::ser::PrettyConfig::new(),
//         )?;

//         // todo - write to file
//         Ok(())
//     }

//     fn load(file: &mut std::fs::File) -> ConfigUIResult<Self> {
//         let mut instr = String::new();
//         file.read_to_string(&mut instr)?;
//         let versioned_config: VersionedConfig = ron::from_str(&instr)?;
//         Ok(ConfigUI::from(versioned_config))
//     }
// }

// #[derive(Debug, Error)]
// pub enum ConfigUIError {
//     #[error("file error: {0}")]
//     FileError(#[from] std::io::Error),
//     #[error("error serializing or deserializing configuration: {0}")]
//     SerdeError(#[from] ron::Error),
//     #[error("error serializing or deserializing configuration: {0}")]
//     SerdeSpannedError(#[from] ron::error::SpannedError),
// }

// pub type ConfigUIResult<T> = Result<T, ConfigUIError>;

// -------------------- Profiles (UI) --------------------
// Like config::Profile, but  uses Vec<LayerUI> instead of Vec<config::Layer>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DefaultProfileUI {
    pub base: settings::BaseLayer,
    pub layers: Vec<LayerUI>,
}

impl Default for DefaultProfileUI {
    fn default() -> Self {
        Self {
            base: settings::BaseLayer::default(),
            layers: Vec::new(),
        }
    }
}

impl From<DefaultProfileUI> for settings::DefaultProfile {
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
//  name
//  enabled
// Also uses Vec<LayerUI> instead of Vec<config::Layer>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProfileUI {
    pub name: String,
    pub enabled: bool,
    pub condition: settings::ProfileCondition,
    pub base: settings::BaseLayer,
    pub layers: Vec<LayerUI>,
}

impl Default for ProfileUI {
    fn default() -> Self {
        Self {
            name: String::from("New Profile"),
            enabled: true,
            condition: settings::ProfileCondition::OriBF,
            base: settings::BaseLayer::default(),
            layers: Vec::new(),
        }
    }
}

impl TryFrom<ProfileUI> for settings::Profile {
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
//  name
//  enabled
// and without these fields:
//  active
// ("enabled" means the user clicked the checkbox for this layer. "active" means the layer is
// currently in effect; for example, the user is holding down the required buttons.)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LayerUI {
    pub name: String,
    pub enabled: bool,
    pub layer_type: settings::LayerType,
    pub condition: Vec<buttons::HoldButton>,
    pub policy: EnumMap<buttons::Button, settings::RemapPolicy>,
}

impl Default for LayerUI {
    fn default() -> Self {
        Self {
            name: String::from("New Layer"),
            enabled: true,
            layer_type: settings::LayerType::Modifier,
            condition: Vec::new(),
            policy: EnumMap::default(),
        }
    }
}

impl TryFrom<LayerUI> for settings::Layer {
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
