use crate::buttons::{Button, HoldButton};
use enum_map::EnumMap;

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
pub struct LayerState {
    pub enabled: bool,
    pub layer_type: LayerType,
    pub condition: Vec<HoldButton>,
    pub policy: EnumMap<Button, RemapPolicy>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProfileState {
    pub base: BaseLayer,
    pub layers: Vec<LayerState>,
}
