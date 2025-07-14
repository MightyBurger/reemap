use super::GuiMenu;
use crate::config;
use crate::gui::reemapp::EditLayerModalOpts;
use crate::gui::reemapp::EditProfileModalOpts;
use crate::gui::reemapp::NewBaseRemapModalOpts;
use crate::gui::reemapp::ProfileConditionUI;
use crate::gui::reemapp::RearrangeLayersModalOpts;
use crate::gui::reemapp::SPACING;
use crate::gui::reemapp::ui_base_layer;
use crate::gui::reemapp::ui_edit_layer_modal::ui_edit_layer_modal;
use crate::gui::reemapp::ui_edit_profile_modal::ui_edit_profile_modal;
use crate::gui::reemapp::ui_ok_cancel_modal::ui_ok_cancel_modal;
use crate::gui::reemapp::ui_tables::ui_enable_clickable_table;
use crate::gui::reemapp::ui_tables::ui_rearrange_table;
use crate::query_windows;

pub fn ui_profile(
    ui: &mut egui::Ui,
    profile: &mut config::Profile,
    rearrange_layers_modal: &mut RearrangeLayersModalOpts,
    edit_profile_modal: &mut EditProfileModalOpts,
    new_layer_modal: &mut EditLayerModalOpts,
    new_base_remap_modal: &mut NewBaseRemapModalOpts,
    profile_idx: usize,
    menu: &mut GuiMenu,
) {
    use egui_extras::{Size, StripBuilder};
    StripBuilder::new(ui)
        .size(Size::relative(0.5))
        .size(Size::remainder())
        .horizontal(|mut strip| {
            strip.cell(|ui| {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    if ui.button("Add Layer").clicked() {
                        *new_layer_modal = EditLayerModalOpts {
                            modal_open: true,
                            name: String::from("New layer"),
                            ..Default::default()
                        };
                    }
                    if ui.button("Rearrange").clicked() {
                        rearrange_layers_modal.new_order = profile.layers.clone();
                        rearrange_layers_modal.modal_open = true;
                    }
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.label(&profile.condition.helper_text());
                        let edit_response = ui.button("Edit");
                        if edit_response.clicked() {
                            *edit_profile_modal = EditProfileModalOpts {
                                modal_open: true,
                                name: profile.name.clone(),
                                condition: match &profile.condition {
                                    // custom
                                    config::ProfileCondition::Always => ProfileConditionUI::Always,
                                    config::ProfileCondition::TitleAndProcess { .. } => {
                                        ProfileConditionUI::TitleAndProcess
                                    }
                                    config::ProfileCondition::Title { .. } => {
                                        ProfileConditionUI::Title
                                    }
                                    config::ProfileCondition::Process { .. } => {
                                        ProfileConditionUI::Process
                                    }
                                    // presets
                                    config::ProfileCondition::OriBF => ProfileConditionUI::OriBF,
                                    config::ProfileCondition::OriBFDE => {
                                        ProfileConditionUI::OriBFDE
                                    }
                                    config::ProfileCondition::OriWotW => {
                                        ProfileConditionUI::OriWotW
                                    }
                                },
                                title: match &profile.condition {
                                    config::ProfileCondition::TitleAndProcess {
                                        title,
                                        process: _,
                                    } => title.clone(),
                                    config::ProfileCondition::Title { title } => title.clone(),
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
                        ui.add_space(super::SPACING);

                        egui::Frame::new()
                            .stroke(egui::Stroke {
                                width: 1.0,
                                color: egui::Color32::DARK_GRAY,
                            })
                            .inner_margin(4.0)
                            .corner_radius(4.0)
                            .show(ui, |ui| {
                                if profile.layers.len() > 0 {
                                    let layer_select =
                                        ui_enable_clickable_table(ui, &mut profile.layers, "Layer");
                                    if let Some(i) = layer_select {
                                        *menu = GuiMenu::ProfileLayer {
                                            profile_idx,
                                            layer_idx: i,
                                        }
                                    }
                                } else {
                                    ui.centered_and_justified(|ui| {
                                        ui.style_mut().interaction.selectable_labels = false;
                                        ui.label("This profile has no layers.");
                                    });
                                }
                            });
                    });
                });
            });
            strip.cell(|ui| {
                ui_base_layer(ui, &mut profile.base, new_base_remap_modal);
            });
        });

    // ----- Rearrange layers modal -----

    if rearrange_layers_modal.modal_open {
        ui_rearrange_layers_modal(ui, rearrange_layers_modal, &mut profile.layers);
    }

    // ----- Edit profile modal -----

    if edit_profile_modal.modal_open {
        let ok_cancel = ui_edit_profile_modal(
            ui,
            edit_profile_modal,
            &format!("Editing profile {}", profile.name),
        );
        match ok_cancel {
            Some(true) => {
                profile.name = edit_profile_modal.clone().name;
                profile.condition = edit_profile_modal.clone().extract_condition();
                edit_profile_modal.modal_open = false;
            }
            Some(false) => {
                edit_profile_modal.modal_open = false;
            }
            None => (),
        }
    }

    // ----- New layer modal -----

    if new_layer_modal.modal_open {
        let ok_cancel = ui_edit_layer_modal(ui, new_layer_modal, "New layer");
        match ok_cancel {
            Some(true) => {
                profile.layers.push(new_layer_modal.clone().into());
                new_layer_modal.modal_open = false;
            }
            Some(false) => {
                new_layer_modal.modal_open = false;
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
    let ok_cancel = ui_ok_cancel_modal(ui, |ui| {
        ui.heading("Rearrange and Delete Layers");
        ui.separator();
        ui.add_space(SPACING);

        egui::Frame::new()
            .stroke(egui::Stroke {
                width: 1.0,
                color: egui::Color32::DARK_GRAY,
            })
            .inner_margin(4.0)
            .corner_radius(4.0)
            .show(ui, |ui| {
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
