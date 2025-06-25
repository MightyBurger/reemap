mod ui_main;
use ui_main::ui_main;

mod ui_profile;
use ui_profile::ui_profile;

mod ui_default_profile;
use ui_default_profile::ui_default_profile;

mod ui_layer;
use ui_layer::ui_layer;

mod ui_base_layer;
use ui_base_layer::ui_base_layer;

mod ui_remap_tables;

mod breadcrumb;
use breadcrumb::breadcrumb;

mod ui_new_remap_modal;

use crate::buttons;
use crate::config;
use crate::hooks;

use enum_map::EnumMap;

const SPACING: f32 = 8.0;

// Thought the name was clever. Don't get too mad, please.
pub struct ReemApp {
    pub hookthread_proxy: hooks::HookthreadProxy,
    pub config: ConfigUI,
    pub gui_local: GuiLocal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum BaseRemapPolicyUI {
    NoRemap,
    Remap,
}

impl Default for BaseRemapPolicyUI {
    fn default() -> Self {
        Self::NoRemap
    }
}

impl std::fmt::Display for BaseRemapPolicyUI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoRemap => write!(f, "No Remap"),
            Self::Remap => write!(f, "Remap"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum RemapPolicyUI {
    Defer,
    NoRemap,
    Remap,
}

impl Default for RemapPolicyUI {
    fn default() -> Self {
        Self::Defer
    }
}

impl std::fmt::Display for RemapPolicyUI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Defer => write!(f, "Defer"),
            Self::NoRemap => write!(f, "No Remap"),
            Self::Remap => write!(f, "Remap"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GuiLocal {
    menu: GuiMenu,
    new_profile_modal_open: bool,
    new_profile: ProfileUI,
    new_layer_modal_open: bool,
    new_layer: LayerUI,
    new_default_layer_modal_open: bool,
    new_default_layer: config::DefaultProfile,
    new_remap_modal: NewRemapModalOpts,
    new_base_remap_modal: NewBaseRemapModalOpts,
}

impl Default for GuiLocal {
    fn default() -> Self {
        Self {
            menu: GuiMenu::default(),
            new_profile_modal_open: false,
            new_profile: ProfileUI::default(),
            new_layer_modal_open: false,
            new_layer: LayerUI::default(),
            new_default_layer_modal_open: false,
            new_default_layer: config::DefaultProfile::default(),
            new_remap_modal: NewRemapModalOpts::default(),
            new_base_remap_modal: NewBaseRemapModalOpts::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NewRemapModalOpts {
    modal_open: Option<buttons::Button>,
    policy: RemapPolicyUI,
    outputs: Vec<buttons::Button>,
}

impl Default for NewRemapModalOpts {
    fn default() -> Self {
        Self {
            modal_open: None,
            policy: RemapPolicyUI::default(),
            outputs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NewBaseRemapModalOpts {
    modal_open: Option<buttons::Button>,
    policy: BaseRemapPolicyUI,
    outputs: Vec<buttons::Button>,
}

impl Default for NewBaseRemapModalOpts {
    fn default() -> Self {
        Self {
            modal_open: None,
            policy: BaseRemapPolicyUI::default(),
            outputs: Vec::new(),
        }
    }
}

// All the possible menus the GUI can be in at any point in time.
// Sure, you could break this into some sort of tree of nested enums.
// But this app has limited scope, and sometimes just solving the problem directly is easier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GuiMenu {
    MainMenu,
    DefaultProfileMenu,
    DefaultProfileBaseLayerMenu,
    DefaultProfileLayerMenu {
        layer_idx: usize,
    },
    ProfileMenu {
        profile_idx: usize,
    },
    ProfileBaseLayerMenu {
        profile_idx: usize,
    },
    ProfileLayerMenu {
        profile_idx: usize,
        layer_idx: usize,
    },
}

impl Default for GuiMenu {
    fn default() -> Self {
        Self::MainMenu
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

// -------------------- Profiles (UI) --------------------
// Like config::Profile, but  uses Vec<LayerUI> instead of Vec<config::Layer>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

// -------------------- Config (UI) --------------------

// Like config::Config, but instantiates ProfileUI instead of Profile and without active_profile
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl crate::gui::TrayApp for ReemApp {
    fn update(&mut self, ctx: &egui::Context) {
        // catppuccin_egui::set_theme(ctx, catppuccin_egui::MACCHIATO);
        egui::TopBottomPanel::bottom("Bottom Panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::Frame::new().inner_margin(2.0).show(ui, |ui| {
                    let left_to_right = egui::Layout {
                        main_dir: egui::Direction::LeftToRight,
                        main_wrap: false,
                        main_align: egui::Align::Min,
                        main_justify: false,
                        cross_align: egui::Align::Center,
                        cross_justify: false,
                    };
                    let right_to_left = egui::Layout {
                        main_dir: egui::Direction::RightToLeft,
                        main_wrap: false,
                        main_align: egui::Align::Min,
                        main_justify: false,
                        cross_align: egui::Align::Center,
                        cross_justify: false,
                    };
                    ui.with_layout(left_to_right, |ui| {
                        ui.label("Reemap");
                    });
                    ui.with_layout(right_to_left, |ui| {
                        if ui.button("Apply").clicked() {
                            self.hookthread_proxy
                                .update(config::Config::from(self.config.clone()));
                        }
                    });
                });
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                breadcrumb(ctx, ui, self);

                ui.separator();
                ui.add_space(SPACING);

                let menu = self.gui_local.menu.clone();
                match menu {
                    GuiMenu::MainMenu => ui_main(ctx, ui, self),
                    GuiMenu::DefaultProfileMenu => ui_default_profile(ctx, ui, self),
                    GuiMenu::DefaultProfileBaseLayerMenu => {
                        let layer = &mut self.config.default.base;
                        ui_base_layer(ctx, ui, layer, &mut self.gui_local.new_base_remap_modal);
                    }
                    GuiMenu::DefaultProfileLayerMenu { layer_idx } => {
                        let layer = &mut self.config.default.layers[layer_idx];
                        ui_layer(ctx, ui, layer, &mut self.gui_local.new_remap_modal);
                    }
                    GuiMenu::ProfileMenu { profile_idx } => ui_profile(ctx, ui, self, profile_idx),
                    GuiMenu::ProfileBaseLayerMenu { profile_idx } => {
                        let layer = &mut self.config.profiles[profile_idx].base;
                        ui_base_layer(ctx, ui, layer, &mut self.gui_local.new_base_remap_modal);
                    }
                    GuiMenu::ProfileLayerMenu {
                        profile_idx,
                        layer_idx,
                    } => {
                        let layer = &mut self.config.profiles[profile_idx].layers[layer_idx];
                        ui_layer(ctx, ui, layer, &mut self.gui_local.new_remap_modal);
                    }
                }
            });
        });
    }
}
