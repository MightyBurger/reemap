mod breadcrumb;
mod ui_base_layer;
mod ui_default_profile;
mod ui_layer;
mod ui_main;
mod ui_new_remap_modal;
mod ui_profile;
mod ui_remap_tables;

use breadcrumb::breadcrumb;
use ui_base_layer::ui_base_layer;
use ui_default_profile::ui_default_profile;
use ui_layer::ui_layer;
use ui_main::ui_main;
use ui_profile::ui_profile;

use crate::config;

use crate::buttons;
use crate::hooks;
use crate::settings;

const SPACING: f32 = 8.0;

// Thought the name was clever. Don't get too mad, please.
pub struct ReemApp {
    pub hookthread_proxy: hooks::HookthreadProxy,
    pub config: config::ConfigUI,
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
    new_profile: config::ProfileUI,
    new_layer_modal_open: bool,
    new_layer: config::LayerUI,
    new_default_layer_modal_open: bool,
    new_default_layer: settings::DefaultProfile,
    new_remap_modal: NewRemapModalOpts,
    new_base_remap_modal: NewBaseRemapModalOpts,
}

impl Default for GuiLocal {
    fn default() -> Self {
        Self {
            menu: GuiMenu::default(),
            new_profile_modal_open: false,
            new_profile: config::ProfileUI::default(),
            new_layer_modal_open: false,
            new_layer: config::LayerUI::default(),
            new_default_layer_modal_open: false,
            new_default_layer: settings::DefaultProfile::default(),
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
                                .update(settings::Settings::from(self.config.clone()));
                        }
                        if ui.button("Test").clicked() {
                            let teststr = ron::ser::to_string_pretty(
                                &self.config,
                                ron::ser::PrettyConfig::default(),
                            );
                            match teststr {
                                Ok(a) => println!("{a}"),
                                Err(e) => println!("{e}"),
                            }
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
