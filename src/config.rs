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
    pub profiles: Vec<Profile>,
    pub show_rare_keys: bool,
}

// -------------------- VersionedProfile --------------------
// A separate versioned Profile is necessary, because profiles can be shared independently.
// Preparing for the future when the Profile struct may change.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum VersionedProfile {
    V1(Profile),
}

impl Default for VersionedProfile {
    fn default() -> Self {
        Self::V1(Profile::default())
    }
}

// Even with future versions, this From<> will be from the latest Profile to VersionedProfile.
impl From<Profile> for VersionedProfile {
    fn from(value: Profile) -> Self {
        Self::V1(value)
    }
}

// In future versions, this From<> will need a little more logic to do migration to the newest
// config version. Right now, as there's only one version of the config, there are no migrations.
impl From<VersionedProfile> for Profile {
    fn from(value: VersionedProfile) -> Self {
        match value {
            VersionedProfile::V1(profile) => profile,
        }
    }
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
            condition: ProfileCondition::OriBFDE,
            base: BaseLayer::default(),
            layers: Vec::new(),
        }
    }
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

// -------------------- ProfileCondition --------------------
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ProfileCondition {
    // custom
    Always,
    TitleAndProcess { title: String, process: String },
    Title { title: String },
    Process { process: String },
    // presets
    OriBF,
    OriBFDE,
    OriWotW,
}

impl ProfileCondition {
    pub fn helper_text(&self) -> String {
        match self {
            Self::Always => "Always active".to_string(),
            Self::TitleAndProcess { title, process } => {
                format!("Active when {title} ({process}) is in focus")
            }
            Self::Title { title } => {
                format!("Active when {title} is in focus")
            }
            Self::Process { process } => {
                format!("Active when the process {process} is in focus")
            }
            Self::OriBF => "Active when Ori and the Blind Forest is in focus".to_string(),
            Self::OriBFDE => {
                "Active when Ori and the Blind Forest: Definitive Edition is in focus".to_string()
            }
            Self::OriWotW => "Active when Ori and the Will of the Wisps is in focus".to_string(),
        }
    }
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

impl std::fmt::Display for Layer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Layer {
    pub fn condition_helper_text(&self) -> String {
        let condition_buttons_str: String = if self.condition.is_empty() {
            String::from("(no buttons set)")
        } else {
            itertools::Itertools::intersperse(
                self.condition.iter().map(|btn| btn.to_string()),
                String::from(", "),
            )
            .collect()
        };
        match self.layer_type {
            LayerType::Modifier => {
                format!("Active when these buttons are held: {condition_buttons_str}")
            }
            LayerType::Toggle => {
                format!("Toggled when these buttons are pressed: {condition_buttons_str}")
            }
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
