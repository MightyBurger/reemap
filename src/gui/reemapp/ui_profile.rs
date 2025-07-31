use super::GuiMenu;
use crate::config;
use crate::gui::reemapp::EditLayerModalOpts;
use crate::gui::reemapp::EditProfileModalOpts;
use crate::gui::reemapp::NewBaseRemapModalOpts;
use crate::gui::reemapp::ProfileConditionUI;
use crate::gui::reemapp::RearrangeLayersModalOpts;
use crate::gui::reemapp::RemapsSearchOpts;
use crate::gui::reemapp::style;
use crate::gui::reemapp::ui_base_layer;
use crate::gui::reemapp::ui_copy_modal::ui_copy_modal;
use crate::gui::reemapp::ui_edit_layer_modal::ui_edit_layer_modal;
use crate::gui::reemapp::ui_edit_profile_modal::ui_edit_profile_modal;
use crate::gui::reemapp::ui_ok_cancel_modal::ui_ok_cancel_modal;
use crate::gui::reemapp::ui_tables::ui_enable_clickable_table;
use crate::gui::reemapp::ui_tables::ui_rearrange_table;
use crate::query_windows;

pub struct UiProfileModals<'a> {
    pub copy_layers_modal: &'a mut bool,
    pub rearrange_layers_modal: &'a mut RearrangeLayersModalOpts,
    pub edit_profile_modal: &'a mut EditProfileModalOpts,
    pub new_layer_modal: &'a mut EditLayerModalOpts,
    pub new_base_remap_modal: &'a mut NewBaseRemapModalOpts,
}

pub fn ui_profile(
    ui: &mut egui::Ui,
    profile: &mut config::Profile,
    profile_idx: usize,
    menu: &mut GuiMenu,
    remaps_search: &mut RemapsSearchOpts,
    show_rare_keys: bool,
    modals: UiProfileModals,
) {
    use crate::gui::reemapp::style::REEMAP_SHADOW;
    use egui_extras::{Size, StripBuilder};

    egui::Frame::new().shadow(REEMAP_SHADOW).show(ui, |ui| {
        StripBuilder::new(ui)
            .size(Size::relative(0.5))
            .size(Size::remainder())
            .horizontal(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .size(Size::remainder())
                        .size(Size::initial(style::BUTTON_HEIGHT))
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                ui.label(profile.condition.helper_text());
                                let edit_response =
                                    ui.add_sized(style::BUTTON_SIZE, egui::Button::new("Edit"));
                                if edit_response.clicked() {
                                    *modals.edit_profile_modal = EditProfileModalOpts {
                                        modal_open: true,
                                        name: profile.name.clone(),
                                        condition: match &profile.condition {
                                            // custom
                                            config::ProfileCondition::Always => {
                                                ProfileConditionUI::Always
                                            }
                                            config::ProfileCondition::TitleAndProcess {
                                                ..
                                            } => ProfileConditionUI::TitleAndProcess,
                                            config::ProfileCondition::Title { .. } => {
                                                ProfileConditionUI::Title
                                            }
                                            config::ProfileCondition::Process { .. } => {
                                                ProfileConditionUI::Process
                                            }
                                        },
                                        title: match &profile.condition {
                                            config::ProfileCondition::TitleAndProcess {
                                                title,
                                                process: _,
                                            } => title.clone(),
                                            config::ProfileCondition::Title { title } => {
                                                title.clone()
                                            }
                                            _ => String::new(),
                                        },
                                        process: match &profile.condition {
                                            config::ProfileCondition::TitleAndProcess {
                                                title: _,
                                                process,
                                            } => process.clone(),
                                            config::ProfileCondition::Process { process } => {
                                                process.clone()
                                            }
                                            _ => String::new(),
                                        },
                                        open_windows: query_windows::enumerate_open_windows(),
                                    };
                                }
                                ui.add_space(style::SPACING);
                                ui.checkbox(&mut profile.clip_cursor, "Confine cursor to window");
                                ui.add_space(style::SPACING);

                                style::UI_FRAME.show(ui, |ui| {
                                    if profile.layers.is_empty() {
                                        ui.centered_and_justified(|ui| {
                                            ui.style_mut().interaction.selectable_labels = false;
                                            ui.label("This profile has no layers.");
                                        });
                                    } else {
                                        let layer_select = ui_enable_clickable_table(
                                            ui,
                                            &mut profile.layers,
                                            "Layer",
                                        );
                                        if let Some(i) = layer_select {
                                            *menu = GuiMenu::ProfileLayer {
                                                profile_idx,
                                                layer_idx: i,
                                            }
                                        }
                                    }
                                });
                            });
                            strip.strip(|builder| {
                                builder
                                    .size(Size::remainder())
                                    .sizes(Size::initial(style::BUTTON_WIDTH), 3) // 3 buttons
                                    .size(Size::remainder())
                                    .horizontal(|mut strip| {
                                        strip.empty();
                                        strip.cell(|ui| {
                                            if ui
                                                .add_sized(
                                                    style::BUTTON_SIZE,
                                                    egui::Button::new("Add Layer"),
                                                )
                                                .clicked()
                                            {
                                                *modals.new_layer_modal = EditLayerModalOpts {
                                                    modal_open: true,
                                                    name: String::from("New layer"),
                                                    ..Default::default()
                                                };
                                            }
                                        });
                                        strip.cell(|ui| {
                                            ui.add_enabled_ui(!profile.layers.is_empty(), |ui| {
                                                if ui
                                                    .add_sized(
                                                        style::BUTTON_SIZE,
                                                        egui::Button::new("Copy Layer"),
                                                    )
                                                    .clicked()
                                                {
                                                    *modals.copy_layers_modal = true;
                                                }
                                            });
                                        });
                                        strip.cell(|ui| {
                                            ui.add_enabled_ui(!profile.layers.is_empty(), |ui| {
                                                if ui
                                                    .add_sized(
                                                        style::BUTTON_SIZE,
                                                        egui::Button::new("Rearrange"),
                                                    )
                                                    .clicked()
                                                {
                                                    modals.rearrange_layers_modal.new_order =
                                                        profile.layers.clone();
                                                    modals.rearrange_layers_modal.modal_open = true;
                                                }
                                            });
                                        });
                                        strip.empty();
                                    });
                            });
                        });
                });
                strip.cell(|ui| {
                    ui_base_layer(
                        ui,
                        &mut profile.base,
                        modals.new_base_remap_modal,
                        remaps_search,
                        show_rare_keys,
                    );
                });
            });
    });

    // ----- Copy layer modal -----

    if *modals.copy_layers_modal {
        ui_copy_modal(ui, modals.copy_layers_modal, &mut profile.layers, "Layer");
    }

    // ----- Rearrange layers modal -----

    if modals.rearrange_layers_modal.modal_open {
        ui_rearrange_layers_modal(ui, modals.rearrange_layers_modal, &mut profile.layers);
    }

    // ----- Edit profile modal -----

    if modals.edit_profile_modal.modal_open {
        let ok_cancel = ui_edit_profile_modal(
            ui,
            modals.edit_profile_modal,
            &format!("Editing profile {}", profile.name),
        );
        match ok_cancel {
            Some(true) => {
                profile.name = modals.edit_profile_modal.clone().name;
                profile.condition = modals.edit_profile_modal.clone().extract_condition();
                modals.edit_profile_modal.modal_open = false;
            }
            Some(false) => {
                modals.edit_profile_modal.modal_open = false;
            }
            None => (),
        }
    }

    // ----- New layer modal -----

    if modals.new_layer_modal.modal_open {
        let ok_cancel =
            ui_edit_layer_modal(ui, modals.new_layer_modal, "New layer", show_rare_keys);
        match ok_cancel {
            Some(true) => {
                profile.layers.push(modals.new_layer_modal.clone().into());
                modals.new_layer_modal.modal_open = false;
            }
            Some(false) => {
                modals.new_layer_modal.modal_open = false;
            }
            None => (),
        }
    }
}

fn ui_rearrange_layers_modal(
    ui: &mut egui::Ui,
    modal_opts: &mut RearrangeLayersModalOpts,
    layers: &mut Vec<config::Layer>,
) {
    let ok_cancel = ui_ok_cancel_modal(ui, "", true, |ui| {
        ui.heading("Rearrange and Delete Layers");
        ui.separator();
        ui.add_space(style::SPACING);

        style::UI_FRAME.show(ui, |ui| {
            ui_rearrange_table(ui, &mut modal_opts.new_order, "Layer");
        });
    });

    match ok_cancel {
        Some(true) => {
            *layers = modal_opts.new_order.clone();
            modal_opts.modal_open = false;
        }
        Some(false) => {
            modal_opts.modal_open = false;
        }
        None => (),
    }
}
