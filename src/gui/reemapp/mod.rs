mod breadcrumb;
mod ui_base_layer;
mod ui_default_profile;
mod ui_edit_profile_modal;
mod ui_layer;
mod ui_main;
mod ui_ok_cancel_modal;
mod ui_profile;
mod ui_status_bar;
mod ui_tables;

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
    new_profile_modal: EditProfileModalOpts,
    edit_profile_modal: EditProfileModalOpts,
    rearrange_profiles_modal: RearrangeProfilesModalOpts,
    new_layer_modal_open: bool,
    new_layer: config::Layer,
    rearrange_layers_modal: RearrangeLayersModalOpts,
    new_default_layer_modal_open: bool,
    new_default_layer: config::DefaultProfile,
    new_remap_modal: NewRemapModalOpts,
    new_base_remap_modal: NewBaseRemapModalOpts,
    layer_condition_modal: LayerConditionModalOpts,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RearrangeProfilesModalOpts {
    modal_open: bool,
    new_order: Vec<config::Profile>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RearrangeLayersModalOpts {
    modal_open: bool,
    new_order: Vec<config::Layer>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EditProfileModalOpts {
    modal_open: bool,
    name: String,
    condition: ProfileConditionUI,
    title: String,
    process: String,
    open_windows: Vec<query_windows::WindowInfo>,
}

impl EditProfileModalOpts {
    fn extract_condition(self) -> config::ProfileCondition {
        match self.condition {
            ProfileConditionUI::TitleAndProcess => config::ProfileCondition::TitleAndProcess {
                title: self.title,
                process: self.process,
            },
            ProfileConditionUI::Title => config::ProfileCondition::Title { title: self.title },
            ProfileConditionUI::Process => config::ProfileCondition::Process {
                process: self.process,
            },
            ProfileConditionUI::OriBF => config::ProfileCondition::OriBF,
            ProfileConditionUI::OriBFDE => config::ProfileCondition::OriBFDE,
            ProfileConditionUI::OriWotW => config::ProfileCondition::OriWotW,
        }
    }
}

impl From<EditProfileModalOpts> for config::Profile {
    fn from(value: EditProfileModalOpts) -> Self {
        Self {
            name: value.name.clone(),
            condition: value.extract_condition(),
            ..Default::default()
        }
    }
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
    DefaultProfileLayer {
        layer_idx: usize,
    },
    Profile {
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
                    GuiMenu::Main => ui_main(ui, self),
                    GuiMenu::DefaultProfile => ui_default_profile(ui, self),
                    GuiMenu::DefaultProfileLayer { layer_idx } => {
                        let layer = &mut self.config.default.layers[layer_idx];
                        ui_layer(
                            ui,
                            layer,
                            &mut self.gui_local.new_remap_modal,
                            &mut self.gui_local.layer_condition_modal,
                        );
                    }
                    GuiMenu::Profile { profile_idx } => ui_profile(ui, self, profile_idx),
                    GuiMenu::ProfileLayer {
                        profile_idx,
                        layer_idx,
                    } => {
                        let layer = &mut self.config.profiles[profile_idx].layers[layer_idx];
                        ui_layer(
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
