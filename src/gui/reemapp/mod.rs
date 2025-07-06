mod breadcrumb;
mod ui_base_layer;
mod ui_default_profile;
mod ui_layer;
mod ui_main;
mod ui_profile;
mod ui_remap_tables;
mod ui_status_bar;

use breadcrumb::breadcrumb;
use std::path::PathBuf;
use tracing::instrument;
use ui_base_layer::ui_base_layer;
use ui_default_profile::ui_default_profile;
use ui_layer::ui_layer;
use ui_main::ui_main;
use ui_profile::ui_profile;

use crate::buttons;
use crate::config;
use crate::config::Output;
use crate::gui::reemapp::ui_status_bar::ui_status_bar;
use crate::hooks;
use crate::query_windows;

const SPACING: f32 = 8.0;

// Thought the name was clever. Don't get too mad, please.
#[derive(Debug)]
pub struct ReemApp {
    pub hookthread_proxy: hooks::HookthreadProxy,
    pub config: config::Config,
    pub gui_local: GuiLocal,
    pub config_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ProfileConditionUI {
    // custom
    TitleAndProcess,
    Title,
    Process,
    // presets
    OriBF,
    OriBFDE,
    OriWotW,
}

impl Default for ProfileConditionUI {
    fn default() -> Self {
        Self::OriBFDE
    }
}

impl std::fmt::Display for ProfileConditionUI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // custom
            Self::TitleAndProcess => write!(f, "Window title and process"),
            Self::Title => write!(f, "Window title"),
            Self::Process => write!(f, "Process"),
            // presets
            Self::OriBF => write!(f, "Ori and the Blind Forest"),
            Self::OriBFDE => write!(f, "Ori and the Blind Forest: Definitive Edition"),
            Self::OriWotW => write!(f, "Ori and the Will of the Wisps"),
        }
    }
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

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GuiLocal {
    menu: GuiMenu,
    new_profile_modal_open: bool,
    new_profile: config::Profile,
    new_layer_modal_open: bool,
    new_layer: config::Layer,
    new_default_layer_modal_open: bool,
    new_default_layer: config::DefaultProfile,
    profile_condition_modal: ProfileConditionModalOpts,
    new_remap_modal: NewRemapModalOpts,
    new_base_remap_modal: NewBaseRemapModalOpts,
    layer_condition_modal: LayerConditionModalOpts,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProfileConditionModalOpts {
    modal_open: bool,
    condition: ProfileConditionUI,
    title: String,
    process: String,
    open_windows: Vec<query_windows::WindowInfo>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NewRemapModalOpts {
    modal_open: Option<buttons::Button>,
    policy: RemapPolicyUI,
    outputs: Output,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayerConditionModalOpts {
    modal_open: bool,
    layer_type: config::LayerType,
    condition: Vec<buttons::HoldButton>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NewBaseRemapModalOpts {
    modal_open: Option<buttons::Button>,
    policy: BaseRemapPolicyUI,
    outputs: Output,
}

// All the possible menus the GUI can be in at any point in time.
// Sure, you could break this into some sort of tree of nested enums.
// But this app has limited scope, and sometimes just solving the problem directly is easier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GuiMenu {
    Main,
    DefaultProfile,
    DefaultProfileBaseLayer,
    DefaultProfileLayer {
        layer_idx: usize,
    },
    Profile {
        profile_idx: usize,
    },
    ProfileBaseLayer {
        profile_idx: usize,
    },
    ProfileLayer {
        profile_idx: usize,
        layer_idx: usize,
    },
}

impl Default for GuiMenu {
    fn default() -> Self {
        Self::Main
    }
}

impl crate::gui::TrayApp for ReemApp {
    #[instrument(skip_all, name = "ui")]
    fn update(&mut self, ctx: &egui::Context) {
        // catppuccin_egui::set_theme(ctx, catppuccin_egui::MACCHIATO);
        egui::TopBottomPanel::bottom("Bottom Panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::Frame::new().inner_margin(2.0).show(ui, |ui| {
                    ui_status_bar(ctx, ui, self);
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
                    GuiMenu::Main => ui_main(ctx, ui, self),
                    GuiMenu::DefaultProfile => ui_default_profile(ctx, ui, self),
                    GuiMenu::DefaultProfileBaseLayer => {
                        let layer = &mut self.config.default.base;
                        ui_base_layer(ctx, ui, layer, &mut self.gui_local.new_base_remap_modal);
                    }
                    GuiMenu::DefaultProfileLayer { layer_idx } => {
                        let layer = &mut self.config.default.layers[layer_idx];
                        ui_layer(
                            ctx,
                            ui,
                            layer,
                            &mut self.gui_local.new_remap_modal,
                            &mut self.gui_local.layer_condition_modal,
                        );
                    }
                    GuiMenu::Profile { profile_idx } => ui_profile(ctx, ui, self, profile_idx),
                    GuiMenu::ProfileBaseLayer { profile_idx } => {
                        let layer = &mut self.config.profiles[profile_idx].base;
                        ui_base_layer(ctx, ui, layer, &mut self.gui_local.new_base_remap_modal);
                    }
                    GuiMenu::ProfileLayer {
                        profile_idx,
                        layer_idx,
                    } => {
                        let layer = &mut self.config.profiles[profile_idx].layers[layer_idx];
                        ui_layer(
                            ctx,
                            ui,
                            layer,
                            &mut self.gui_local.new_remap_modal,
                            &mut self.gui_local.layer_condition_modal,
                        );
                    }
                }
            });
        });
    }
}
