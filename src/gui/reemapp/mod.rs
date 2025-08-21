// Copyright 2025 Jordan Johnson
// SPDX-License-Identifier: Apache-2.0 OR MIT

mod breadcrumb;
mod style;
mod ui_base_layer;
mod ui_copy_modal;
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
use tracing::{error, info, instrument};
use ui_base_layer::ui_base_layer;
use ui_layer::ui_layer;
use ui_main::ui_main;
use ui_profile::ui_profile;

use crate::buttons;
use crate::config;
use crate::config::Output;
use crate::gui::TrayAppCtx;
use crate::gui::reemapp::ui_profile::UiProfileModals;
use crate::hooks;
use crate::query_windows;

use windows::Win32::UI::Input::KeyboardAndMouse;

// Thought the name was clever. Don't get too mad, please.
#[derive(Debug)]
pub struct ReemApp {
    hookthread_proxy: hooks::HookthreadProxy,
    config: config::Config,
    current_config: config::Config,
    schedule_discard: bool,
    config_path: PathBuf,
    gui_local: GuiLocal,
}

impl ReemApp {
    pub fn new(
        hookthread_proxy: hooks::HookthreadProxy,
        config: config::Config,
        config_path: PathBuf,
    ) -> Self {
        Self {
            hookthread_proxy,
            current_config: config.clone(),
            config,
            schedule_discard: false,
            config_path,
            gui_local: GuiLocal::default(),
        }
    }
    fn apply_changes(&mut self) {
        // Three things happen when setting the configuration.
        // 1. UI therad saves configuration to %APPDATA%
        // 2. UI thread sends config over to hookthread to update the remaps
        // 3. UI thread updates its own current_config value

        let config_str = ron::ser::to_string_pretty(
            &config::VersionedConfig::from(self.config.clone()),
            ron::ser::PrettyConfig::new(),
        )
        .unwrap();
        match std::fs::write(&self.config_path, config_str) {
            Ok(()) => (),
            Err(e) => {
                error!("could not write to config file: {e}");
                native_dialog::DialogBuilder::message()
                    .set_level(native_dialog::MessageLevel::Error)
                    .set_title("Error writing file")
                    .set_text("Reemap could not write to the configuration file.")
                    .alert()
                    .show()
                    .unwrap();
            }
        }

        self.hookthread_proxy.update(self.config.clone());

        self.current_config = self.config.clone();
    }
    fn discard_changes(&mut self) {
        self.schedule_discard = true;
    }
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
        Self::TitleAndProcess
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
    Suppress,
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
            Self::Suppress => write!(f, "Suppress"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum RemapPolicyUI {
    Defer,
    NoRemap,
    Remap,
    Suppress,
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
            Self::Suppress => write!(f, "Suppress"),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct GuiLocal {
    menu: GuiMenu,
    remaps_search_base: RemapsSearchOpts,
    remaps_search_layer: RemapsSearchOpts,
    new_profile_modal: EditProfileModalOpts,
    edit_profile_modal: EditProfileModalOpts,
    copy_profile_modal: bool,
    rearrange_profiles_modal: RearrangeProfilesModalOpts,
    new_layer_modal: EditLayerModalOpts,
    edit_layer_modal: EditLayerModalOpts,
    copy_layer_modal: bool,
    rearrange_layers_modal: RearrangeLayersModalOpts,
    new_remap_modal: NewRemapModalOpts,
    new_base_remap_modal: NewBaseRemapModalOpts,
    see_buttons_modal: bool,
    about_modal: bool,
    settings_modal: SettingsModalOpts,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemapsSearchOpts {
    search_string: String,
    hide_unmapped: bool,
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

#[derive(Debug, Default, Clone, PartialEq)]
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
            ProfileConditionUI::OriBF => config::ProfileCondition::TitleAndProcess {
                title: "Ori And The Blind Forest".to_string(),
                process: "ori.exe".to_string(),
            },
            ProfileConditionUI::OriBFDE => config::ProfileCondition::TitleAndProcess {
                title: "Ori And The Blind Forest: Definitive Edition".to_string(),
                process: "oriDE.exe".to_string(),
            },
            ProfileConditionUI::OriWotW => config::ProfileCondition::TitleAndProcess {
                title: "OriAndTheWilloftheWisps".to_string(),
                process: "oriwotw.exe".to_string(),
            },
        }
    }
    fn valid(&self) -> bool {
        !self.name.is_empty()
            && match self.condition {
                ProfileConditionUI::TitleAndProcess => {
                    !self.title.is_empty() && !self.process.is_empty()
                }
                ProfileConditionUI::Title => !self.title.is_empty(),
                ProfileConditionUI::Process => !self.process.is_empty(),
                ProfileConditionUI::Always
                | ProfileConditionUI::OriBF
                | ProfileConditionUI::OriBFDE
                | ProfileConditionUI::OriWotW => true,
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
    search: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EditLayerModalOpts {
    modal_open: bool,
    name: String,
    layer_type: config::LayerType,
    condition: Vec<buttons::HoldButton>,
    search: String,
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
    search: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SettingsModalOpts {
    modal_open: bool,
    // None means there was an error trying to access the registry.
    run_on_login: Option<bool>,
    background: config::Background,
    current_run_on_login: Option<bool>,
    show_rare_keys: bool,
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
    fn update(&mut self, ctx: &egui::Context, app_ctx: &TrayAppCtx) {
        let TrayAppCtx {
            last_pressed_button,
        } = app_ctx;

        ctx.set_visuals(egui::Visuals::dark());

        // This is a lot less falliable than having a "dirty" flag set every time a setting
        // changes. Yet, it is probably a lot less performant. This could be optimized.
        let unsaved_changes = self.current_config != self.config;

        egui_extras::install_image_loaders(ctx);

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
                                if self.config.profiles.is_empty() {
                                    ui.label("(no profiles)");
                                } else {
                                    for profile in self.config.profiles.iter() {
                                        if ui.button(profile.name.clone()).clicked() {
                                            export_profile_dialog(profile.clone());
                                        }
                                    }
                                }
                            });
                        });
                        ui.separator();
                        if ui.button("Settings").clicked() {
                            if unsaved_changes {
                                native_dialog::DialogBuilder::message()
                                    .set_level(native_dialog::MessageLevel::Info)
                                    .set_title("Unsaved changes")
                                    .set_text("You have unsaved changes. Apply or discard changes before accessing settings.")
                                    .alert()
                                    .show()
                                    .unwrap();
                            } else {
                                let run_on_login = match crate::registry::is_registered_run_on_login()
                                 {
                                    Ok(val) => Some(val),
                                    Err(e) => {
                                            native_dialog::DialogBuilder::message()
                                                .set_level(native_dialog::MessageLevel::Warning)
                                                .set_title("Error accessing registry")
                                                .set_text(format!("Reemap could not access the registry. The run-on-login setting will be unavailable.\n\n{e}"))
                                                .alert()
                                                .show()
                                                .unwrap();
                                            warn!("error accessing registry: {e}");
                                        None
                                    }
                                };
                                self.gui_local.settings_modal = SettingsModalOpts {
                                    modal_open: true,
                                    run_on_login,
                                    background: self.config.background,
                                    current_run_on_login: run_on_login,
                                    show_rare_keys: self.config.show_rare_keys
                                };
                            }
                        }
                    });
                    ui.menu_button("Tools", |ui| {
                        if ui.button("Button Viewer").clicked() {
                            self.gui_local.see_buttons_modal = true;
                            self.hookthread_proxy.register_observe_inputs();
                        }
                    });
                    ui.menu_button("Help", |ui| {
                        ui.hyperlink_to("User's Guide", "https://github.com/MightyBurger/reemap");
                        if ui.button("About").clicked() {
                            self.gui_local.about_modal = true;
                        }
                    });
                });
            });

        // Display a message to inform the user if remaps are disabled
        if unsafe { KeyboardAndMouse::GetKeyState(KeyboardAndMouse::VK_SCROLL.0.into()) & 1 > 0 } {
            let warning_frame = egui::Frame::new().fill(egui::Color32::DARK_RED);
            egui::TopBottomPanel::bottom("ui_warn_panel")
                .frame(warning_frame)
                .show(ctx, |ui| {
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::BottomUp),
                        |ui| {
                            ui.strong("Remaps are disabled because Scroll Lock is on!");
                        },
                    );
                });
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(0, 0))
                    .fill(egui::Color32::BLACK),
            )
            .show(ctx, |ui| {
                // Show a background image
                match self.config.background {
                    config::Background::Ginso => {
                        egui::Image::new(egui::include_image!("../../../resource/background.png"))
                            .tint(egui::Color32::from_gray(64))
                            .paint_at(ui, [[0.0, 0.0].into(), [800.0, 600.0].into()].into());
                    }
                    config::Background::Gradient => {
                        egui::Image::new(egui::include_image!("../../../resource/gradient.svg"))
                            .paint_at(ui, [[0.0, 0.0].into(), [800.0, 600.0].into()].into());
                    }
                }
                egui::Frame::new().inner_margin(12.0).show(ui, |ui| {
                    style::set_reemap_style(ui);

                    breadcrumb(ctx, ui, self, unsaved_changes);
                    ui.separator();
                    ui.add_space(style::SPACING);

                    let menu = self.gui_local.menu.clone();
                    match menu {
                        GuiMenu::Main => ui_main(ui, self),
                        GuiMenu::Profile { profile_idx } => ui_profile(
                            ui,
                            &mut self.config.profiles[profile_idx],
                            profile_idx,
                            &mut self.gui_local.menu,
                            &mut self.gui_local.remaps_search_base,
                            self.config.show_rare_keys,
                            UiProfileModals {
                                copy_layers_modal: &mut self.gui_local.copy_layer_modal,
                                rearrange_layers_modal: &mut self.gui_local.rearrange_layers_modal,
                                edit_profile_modal: &mut self.gui_local.edit_profile_modal,
                                new_layer_modal: &mut self.gui_local.edit_layer_modal,
                                new_base_remap_modal: &mut self.gui_local.new_base_remap_modal,
                            },
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
                                &mut self.gui_local.remaps_search_layer,
                                self.config.show_rare_keys,
                            );
                        }
                    }
                });

                if self.gui_local.settings_modal.modal_open {
                    settings_modal(ui, self);
                }
                if self.gui_local.see_buttons_modal {
                    see_buttons_modal(
                        ui,
                        &mut self.gui_local.see_buttons_modal,
                        &self.hookthread_proxy,
                        *last_pressed_button,
                    );
                }
                if self.gui_local.about_modal {
                    about_modal(ui, &mut self.gui_local.about_modal);
                }
            });
        if self.schedule_discard {
            self.schedule_discard = false;
            self.config = self.current_config.clone();
            self.gui_local.menu = GuiMenu::Main;
        }
    }
}

fn settings_modal(ui: &mut egui::Ui, args: &mut ReemApp) {
    use ui_ok_cancel_modal::ui_ok_cancel_modal;

    let modal_opts = &mut args.gui_local.settings_modal;

    let ok_cancel = ui_ok_cancel_modal(ui, "", true, |ui| {
        ui.heading("Reemap Settings");
        ui.separator();
        ui.add_space(style::SPACING);
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Background
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("settings_background_combo")
                    .selected_text(format!("{}", &modal_opts.background))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut modal_opts.background,
                            config::Background::Ginso,
                            "Ginso Tree",
                        );
                        ui.selectable_value(
                            &mut modal_opts.background,
                            config::Background::Gradient,
                            "Gradient",
                        );
                    });
                ui.strong("Background");
            });
            ui.add_space(style::SPACING);
            ui.label("Choose which image to display in the background of Reemap.");
            ui.add_space(style::SPACING);
            ui.separator();
            ui.add_space(style::SPACING);

            // run on login
            if let Some(run_on_login) = modal_opts.run_on_login.as_mut() {
                ui.checkbox(run_on_login, "Run on login");
            } else {
                let mut dummy = false;
                ui.add_enabled_ui(false, |ui| {
                    ui.checkbox(&mut dummy, "Run on login");
                });
            }
            ui.add_space(style::SPACING);
            ui.label(
                "When checked, Reemap will start when you log in. This setting only affects \
you; it will not affect other users on this computer.

This means remaps will apply as soon as you log in. Be careful if you have a profile \
that runs unconditionally. If you get yourself stuck, remember: you can enable scroll lock to \
disable remaps!",
            );
            ui.add_space(style::SPACING);
            ui.separator();
            ui.add_space(style::SPACING);

            // show unusual keys
            ui.checkbox(&mut modal_opts.show_rare_keys, "Show unusual keys");
            ui.add_space(style::SPACING);
            ui.label(
            "Unusual keyboard keys include keys that are uncommon in modern hardware and keys you \
probably do not want to remap. Examples include \"mouse-button-as-key\" keys and \
Input Method Editor (IME) keys. Remaps may behave strangely depending on the key. Check this box \
if you need to remap these keys.

Note: even with this setting enabled, some keys are unavailable. This includes every key \
Windows defines as reserved, undefined, or unassigned. This also includes the Scroll \
Lock key, which Reemap uses as an escape-hatch to disable all remaps.",
        );
        });
    });
    match ok_cancel {
        Some(true) => {
            if modal_opts.current_run_on_login != modal_opts.run_on_login
                && let Some(run) = modal_opts.run_on_login
            {
                let result = crate::registry::run_on_login(run);
                if let Err(e) = result {
                    native_dialog::DialogBuilder::message()
                        .set_level(native_dialog::MessageLevel::Warning)
                        .set_title("Error accessing registry")
                        .set_text(format!("Reemap could not access the registry. The run-on-login setting was not applied.\n\n{e}"))
                        .alert()
                        .show()
                        .unwrap();
                    warn!("error accessing registry: {e}");
                }
            }

            args.config.background = modal_opts.background;
            args.config.show_rare_keys = modal_opts.show_rare_keys;
            modal_opts.modal_open = false;
            args.apply_changes();
        }
        Some(false) => {
            modal_opts.modal_open = false;
        }
        None => (),
    }
}

fn see_buttons_modal(
    ui: &mut egui::Ui,
    modal_opts: &mut bool,
    hookthread_proxy: &crate::hooks::HookthreadProxy,
    last_pressed_button: Option<crate::buttons::Button>,
) {
    use buttons::key::KeyType;
    let modal = egui::Modal::new(egui::Id::new("about modal"))
        .backdrop_color(style::MODAL_BACKDROP_COLOR)
        .frame(style::MODAL_FRAME)
        .show(ui.ctx(), |ui| {
            style::set_reemap_style(ui);

            ui.heading("Button Viewer");

            ui.add_space(style::SPACING);
            ui.label("Click away from Reemap so it is not in focus.");
            ui.add_space(style::SPACING);
            ui.label("The last pressed button is: ");
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.add_space(style::SPACING);
                ui.strong(match last_pressed_button {
                    None => "(none)".to_string(),
                    Some(button) => button.to_string(),
                });
                if let Some(buttons::Button::Key(key)) = last_pressed_button
                    && key.key_type() == KeyType::Rare
                {
                    ui.add_space(style::SPACING);
                    ui.label("To remap this key, you must enable unusual keys in Reemap settings.");
                }
                if let Some(buttons::Button::Key(key)) = last_pressed_button
                    && key.key_type() == KeyType::Unmappable
                {
                    ui.add_space(style::SPACING);
                    ui.label("This key cannot be remapped.");
                }
            });
        });
    if modal.should_close() {
        *modal_opts = false;
        hookthread_proxy.unregister_observe_inputs();
    }
}

fn about_modal(ui: &mut egui::Ui, modal_opts: &mut bool) {
    use egui::special_emojis::GITHUB;

    let modal = egui::Modal::new(egui::Id::new("about modal"))
        .backdrop_color(style::MODAL_BACKDROP_COLOR)
        .frame(style::MODAL_FRAME)
        .show(ui.ctx(), |ui| {
            style::set_reemap_style(ui);
            let version = env!("CARGO_PKG_VERSION");
            ui.heading("Reemap");
            ui.label(version);
            ui.add_space(style::SPACING);
            ui.label("Reemap is an input remapping tool.");
            ui.add_space(style::SPACING);
            ui.label("Reemap is free to use. The source code is available under a permissive license. See the repository for more information.");
            ui.add_space(style::SPACING);
            ui.hyperlink_to(format!("{GITHUB} Reemap on Github"), "https://github.com/MightyBurger/reemap");
            ui.add_space(style::SPACING);
            ui.label("Copyright © 2025 Jordan Johnson");
        });
    if modal.should_close() {
        *modal_opts = false;
    }
}

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
