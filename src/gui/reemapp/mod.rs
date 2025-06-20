mod ui_main;
use ui_main::ui_main;

mod ui_profile;
use ui_profile::ui_profile;

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

impl ReemApp {
    fn get_open_profile_ui(&mut self) -> Option<&mut ProfileUI> {
        match self.gui_local.menu {
            GuiMenu::MainMenu => None,
            GuiMenu::DefaultProfileMenu => None,
            GuiMenu::ProfileMenu { profile_idx } => Some(&mut self.config.profiles[profile_idx]),
            GuiMenu::BaseLayerMenu { profile_idx } => Some(&mut self.config.profiles[profile_idx]),
            GuiMenu::LayerMenu {
                profile_idx,
                layer_idx: _,
            } => Some(&mut self.config.profiles[profile_idx]),
        }
    }
    fn get_open_profile_ui_idx(&self) -> Option<usize> {
        match self.gui_local.menu {
            GuiMenu::MainMenu => None,
            GuiMenu::DefaultProfileMenu => None,
            GuiMenu::ProfileMenu { profile_idx } => Some(profile_idx),
            GuiMenu::BaseLayerMenu { profile_idx } => Some(profile_idx),
            GuiMenu::LayerMenu {
                profile_idx,
                layer_idx: _,
            } => Some(profile_idx),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GuiLocal {
    menu: GuiMenu,
    new_profile_modal_open: bool,
    new_profile: ProfileUI,
}

impl Default for GuiLocal {
    fn default() -> Self {
        Self {
            menu: GuiMenu::default(),
            new_profile_modal_open: false,
            new_profile: ProfileUI::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GuiMenu {
    MainMenu,
    DefaultProfileMenu,
    ProfileMenu {
        profile_idx: usize,
    },
    BaseLayerMenu {
        profile_idx: usize,
    },
    LayerMenu {
        profile_idx: usize,
        layer_idx: usize,
    },
}

impl Default for GuiMenu {
    fn default() -> Self {
        Self::MainMenu
    }
}

// Like config::Config, but with extra fields:
//  enabled
//  name
// and without these fields:
//  active
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayerUI {
    // Checked in the UI?
    pub enabled: bool,
    pub layer_type: config::LayerType,
    pub condition: Vec<buttons::HoldButton>,
    pub policy: EnumMap<buttons::Button, config::RemapPolicy>,
    pub name: String,
}

impl From<LayerUI> for config::Layer {
    fn from(value: LayerUI) -> Self {
        Self {
            active: false,
            layer_type: value.layer_type,
            condition: value.condition,
            policy: value.policy,
        }
    }
}

// Like config::Profile, but with extra fields:
// enabled
//  name
//  condition
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProfileUI {
    pub enabled: bool,
    pub condition: config::ProfileCondition,
    pub base: config::BaseLayer,
    pub layers: Vec<LayerUI>,
    pub name: String,
}

impl Default for ProfileUI {
    fn default() -> Self {
        Self {
            enabled: true,
            condition: config::ProfileCondition::OriBF,
            base: config::BaseLayer::default(),
            layers: Vec::new(),
            name: String::from("New Profile"),
        }
    }
}

impl From<ProfileUI> for config::Profile {
    fn from(value: ProfileUI) -> Self {
        Self {
            base: value.base,
            layers: value
                .layers
                .into_iter()
                .map(|layer_ui| layer_ui.into())
                .collect(),
        }
    }
}

// Like config::Config, but instantiates ProfileUI instead of Profile and without active_profile
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConfigUI {
    pub default: config::Profile,
    pub profiles: Vec<ProfileUI>,
}

impl From<ConfigUI> for config::Config {
    fn from(value: ConfigUI) -> Self {
        Self {
            default: value.default,
            profiles: value
                .profiles
                .iter()
                .cloned()
                .map(|profiles_ui| profiles_ui.into())
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
            let menu = self.gui_local.menu.clone();
            match menu {
                GuiMenu::MainMenu => ui_main(ctx, ui, self),
                GuiMenu::ProfileMenu { profile_idx } => ui_profile(ctx, ui, self),
                _ => (),
            }
        });
    }
}
