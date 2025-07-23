mod breadcrumb;
mod style;
mod ui_base_layer;
mod ui_edit_layer_modal;
mod ui_edit_profile_modal;
mod ui_layer;
mod ui_main;
mod ui_ok_cancel_modal;
mod ui_profile;
mod ui_tables;

use breadcrumb::breadcrumb;
use std::path::PathBuf;
use tracing::warn;
use tracing::{info, instrument};
use ui_base_layer::ui_base_layer;
use ui_layer::ui_layer;
use ui_main::ui_main;
use ui_profile::ui_profile;

use crate::buttons;
use crate::config;
use crate::config::Output;
use crate::hooks;
use crate::query_windows;

const SPACING: f32 = 8.0;
const HEADER_HEIGHT: f32 = 12.0;
const ROW_HEIGHT: f32 = 20.0;
const BUTTON_WIDTH: f32 = 80.0;
const BUTTON_HEIGHT: f32 = 20.0;
const BUTTON_SIZE: [f32; 2] = [BUTTON_WIDTH, BUTTON_HEIGHT];

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
    Always,
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
            Self::Always => write!(f, "Always active"),
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
    new_layer_modal: EditLayerModalOpts,
    edit_layer_modal: EditLayerModalOpts,
    rearrange_layers_modal: RearrangeLayersModalOpts,
    new_remap_modal: NewRemapModalOpts,
    new_base_remap_modal: NewBaseRemapModalOpts,
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
            ProfileConditionUI::Always => config::ProfileCondition::Always,
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
pub struct EditLayerModalOpts {
    modal_open: bool,
    name: String,
    layer_type: config::LayerType,
    condition: Vec<buttons::HoldButton>,
}

impl From<EditLayerModalOpts> for config::Layer {
    fn from(value: EditLayerModalOpts) -> Self {
        Self {
            name: value.name.clone(),
            layer_type: value.layer_type.clone(),
            condition: value.condition.clone(),
            ..Default::default()
        }
    }
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
        egui_extras::install_image_loaders(ctx);
        // catppuccin_egui::set_theme(ctx, catppuccin_egui::MACCHIATO);

        egui::TopBottomPanel::top("menu bar panel")
            .resizable(false)
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Import Profile").clicked() {
                            if let Some(profile) = import_profile_dialog() {
                                self.config.profiles.push(profile);
                            }
                        }
                        ui.menu_button("Export Profile", |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.set_min_width(200.0);
                                for profile in self.config.profiles.iter() {
                                    if ui.button(profile.name.clone()).clicked() {
                                        export_profile_dialog(profile.clone());
                                    }
                                }
                            });
                        });
                    });
                    ui.menu_button("Help", |ui| {
                        if ui.button("About").clicked() {
                            todo!();
                        }
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(0, 0))
                    .fill(egui::Color32::BLACK),
            )
            .show(ctx, |ui| {
                egui::Image::new(egui::include_image!("../../../resource/background.png"))
                    .tint(egui::Color32::from_gray(64))
                    .paint_at(ui, [[0.0, 0.0].into(), [800.0, 600.0].into()].into());
                egui::Frame::new().inner_margin(12.0).show(ui, |ui| {
                    style::set_reemap_style(ui);

                    breadcrumb(ctx, ui, self);
                    ui.separator();
                    ui.add_space(SPACING);

                    let menu = self.gui_local.menu.clone();
                    match menu {
                        GuiMenu::Main => ui_main(ui, self),
                        GuiMenu::Profile { profile_idx } => ui_profile(
                            ui,
                            &mut self.config.profiles[profile_idx],
                            &mut self.gui_local.rearrange_layers_modal,
                            &mut self.gui_local.edit_profile_modal,
                            &mut self.gui_local.edit_layer_modal,
                            &mut self.gui_local.new_base_remap_modal,
                            profile_idx,
                            &mut self.gui_local.menu,
                        ),
                        GuiMenu::ProfileLayer {
                            profile_idx,
                            layer_idx,
                        } => {
                            let layer = &mut self.config.profiles[profile_idx].layers[layer_idx];
                            ui_layer(
                                ui,
                                layer,
                                &mut self.gui_local.new_remap_modal,
                                &mut self.gui_local.edit_layer_modal,
                            );
                        }
                    }
                });
            });
    }
}

// style::set_reemap_style(ui);

// StripBuilder::new(ui)
//     .size(Size::initial(40.0))
//     .size(Size::remainder())
//     .vertical(|mut strip| {
//         strip.cell(|ui| {
//             breadcrumb(ctx, ui, self);
//             // ui.separator();
//             // ui.add_space(SPACING);
//         });
//         strip.cell(|ui| {
//             info!("available: {:?}", ui.available_size());
//             let menu = self.gui_local.menu.clone();
//             match menu {
//                 GuiMenu::Main => ui_main(ui, self),
//                 GuiMenu::Profile { profile_idx } => ui_profile(
//                     ui,
//                     &mut self.config.profiles[profile_idx],
//                     &mut self.gui_local.rearrange_layers_modal,
//                     &mut self.gui_local.edit_profile_modal,
//                     &mut self.gui_local.edit_layer_modal,
//                     &mut self.gui_local.new_base_remap_modal,
//                     profile_idx,
//                     &mut self.gui_local.menu,
//                 ),
//                 GuiMenu::ProfileLayer {
//                     profile_idx,
//                     layer_idx,
//                 } => {
//                     let layer = &mut self.config.profiles[profile_idx].layers
//                         [layer_idx];
//                     ui_layer(
//                         ui,
//                         layer,
//                         &mut self.gui_local.new_remap_modal,
//                         &mut self.gui_local.edit_layer_modal,
//                     );
//                 }
//             }
//         });
//     });

fn import_profile_dialog() -> Option<config::Profile> {
    fn display_warning(text: &str, ctx: impl std::fmt::Display) {
        let body_text = format!("{text}\n\n{ctx}");
        native_dialog::DialogBuilder::message()
            .set_level(native_dialog::MessageLevel::Error)
            .set_title("Error importing profile")
            .set_text(&body_text)
            .alert()
            .show()
            .unwrap();
        warn!("error opening profile: {}", &body_text);
    }

    let selection = native_dialog::DialogBuilder::file()
        .add_filter("RON", ["ron"])
        .open_single_file()
        .show();
    let selection = match selection {
        Ok(Some(path)) => path,
        Ok(None) => return None,
        Err(e) => {
            display_warning("Error with file selection dialog.", e);
            return None;
        }
    };

    let profile_str = match std::fs::read_to_string(selection) {
        Ok(profile_str) => profile_str,
        Err(e) => {
            display_warning("Error opening file.", e);
            return None;
        }
    };

    let versioned_profile: config::VersionedProfile = match ron::from_str(&profile_str) {
        Ok(prf) => prf,
        Err(e) => {
            display_warning(
                "Error parsing profile. Was this profile made in a newer version of Reemap?",
                e,
            );
            return None;
        }
    };

    let profile = config::Profile::from(versioned_profile);
    native_dialog::DialogBuilder::message()
        .set_level(native_dialog::MessageLevel::Info)
        .set_title("Imported profile")
        .set_text(format!("Imported profile {}", &profile.name))
        .alert()
        .show()
        .unwrap();
    info!("imported profile {}", &profile.name);

    Some(profile)
}

fn export_profile_dialog(profile: config::Profile) {
    let name = profile.name.clone();
    let selection = native_dialog::DialogBuilder::file()
        .add_filter("RON", ["ron"])
        .set_filename(&name)
        .save_single_file()
        .show();
    let versioned_profile = config::VersionedProfile::from(profile);
    match selection {
        Ok(None) => (),
        Ok(Some(path)) => {
            let profile_str =
                ron::ser::to_string_pretty(&versioned_profile, ron::ser::PrettyConfig::new())
                    .unwrap();
            match std::fs::write(&path, profile_str) {
                Ok(()) => {
                    native_dialog::DialogBuilder::message()
                        .set_level(native_dialog::MessageLevel::Info)
                        .set_title("Exported profile")
                        .set_text(format!(
                            "Exported profile {} to {}",
                            &name,
                            path.to_str().unwrap_or("(path not UTF-8)")
                        ))
                        .alert()
                        .show()
                        .unwrap();
                    info!("exported profile");
                }
                Err(e) => {
                    native_dialog::DialogBuilder::message()
                        .set_level(native_dialog::MessageLevel::Warning)
                        .set_title("Error exporting profile")
                        .set_text(format!("Reemap could not export the profile.\n\n{e}"))
                        .alert()
                        .show()
                        .unwrap();
                    warn!(?e, "failed to export profile");
                }
            }
        }
        Err(e) => warn!(?e, "error opening export dialog"),
    }
}
