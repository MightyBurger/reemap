use crate::buttons;
use enum_map::EnumMap;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

pub const REMAP_SMALLVEC_LEN: usize = 8;
pub type Output = SmallVec<[buttons::Button; REMAP_SMALLVEC_LEN]>;

// -------------------- VersionedConfig --------------------
// Preparing for the future when the Config struct may change.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum VersionedConfig {
    V1(Config),
}

impl Default for VersionedConfig {
    fn default() -> Self {
        Self::V1(Config::default())
    }
}

// Even with future versions, this From<> will be from the latest Config to VersionedConfig.
impl From<Config> for VersionedConfig {
    fn from(value: Config) -> Self {
        Self::V1(value)
    }
}

// In future versions, this From<> will need a little more logic to do migration to the newest
// config version. Right now, as there's only one version of the config, there are no migrations.
impl From<VersionedConfig> for Config {
    fn from(value: VersionedConfig) -> Self {
        match value {
            VersionedConfig::V1(config_ui) => config_ui,
        }
    }
}

// -------------------- Config --------------------
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Config {
    pub default: DefaultProfile,
    pub profiles: Vec<Profile>,
}

// -------------------- DefaultProfile --------------------
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DefaultProfile {
    pub base: BaseLayer,
    pub layers: Vec<Layer>,
}

// -------------------- Profile --------------------
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub enabled: bool,
    pub condition: ProfileCondition,
    pub base: BaseLayer,
    pub layers: Vec<Layer>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: String::from("New Profile"),
            enabled: true,
            condition: ProfileCondition::OriBF,
            base: BaseLayer::default(),
            layers: Vec::new(),
        }
    }
}

// -------------------- ProfileCondition --------------------
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ProfileCondition {
    // presets
    OriBF,
    OriBFDE,
    OriWotW,
    // custom
    TitleAndProcess { title: String, process: String },
    Title { title: String },
    Process { process: String },
}

// -------------------- BaseLayer --------------------
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BaseLayer {
    pub policy: EnumMap<buttons::Button, BaseRemapPolicy>,
}

// -------------------- Layers --------------------
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub enabled: bool,
    pub layer_type: LayerType,
    pub condition: Vec<buttons::HoldButton>,
    pub policy: EnumMap<buttons::Button, RemapPolicy>,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            name: String::from("New Layer"),
            enabled: true,
            layer_type: LayerType::default(),
            condition: Vec::new(),
            policy: EnumMap::default(),
        }
    }
}

// -------------------- LayerType --------------------
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LayerType {
    Modifier,
    Toggle,
}

impl std::fmt::Display for LayerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Modifier => write!(f, "Modifier"),
            Self::Toggle => write!(f, "Toggle"),
        }
    }
}

impl Default for LayerType {
    fn default() -> Self {
        Self::Modifier
    }
}

// -------------------- BaseRemapPolicy --------------------
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BaseRemapPolicy {
    NoRemap,
    Remap(Output),
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
                if maps.is_empty() {
                    return write!(f, "(block input)");
                }
                let outstr: String = itertools::Itertools::intersperse(
                    maps.iter().map(|btn| btn.to_string()),
                    String::from(", "),
                )
                .collect();
                write!(f, "{outstr}")
            }
        }
    }
}

// -------------------- RemapPolicy --------------------
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RemapPolicy {
    Defer,
    NoRemap,
    Remap(Output),
}

impl std::fmt::Display for RemapPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Defer => write!(f, "(defer to next layer)"),
            Self::NoRemap => write!(f, "(do not remap)"),
            Self::Remap(maps) => {
                if maps.is_empty() {
                    return write!(f, "(block input)");
                }
                let outstr: String = itertools::Itertools::intersperse(
                    maps.iter().map(|btn| btn.to_string()),
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
