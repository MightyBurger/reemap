use super::GuiMenu;
use super::ReemApp;
use crate::config;
use crate::gui::reemapp::EditProfileModalOpts;
use crate::gui::reemapp::ProfileConditionUI;
use crate::gui::reemapp::RearrangeLayersModalOpts;
use crate::gui::reemapp::SPACING;
use crate::gui::reemapp::ui_base_layer;
use crate::gui::reemapp::ui_edit_profile_modal::ui_edit_profile_modal;
use crate::gui::reemapp::ui_ok_cancel_modal::ui_ok_cancel_modal;
use crate::gui::reemapp::ui_tables::ui_enable_clickable_table;
use crate::gui::reemapp::ui_tables::ui_rearrange_table;
use crate::query_windows;

pub fn ui_profile(ui: &mut egui::Ui, args: &mut ReemApp, profile_idx: usize) {
    use egui_extras::{Size, StripBuilder};
    StripBuilder::new(ui)
        .size(Size::relative(0.5))
        .size(Size::remainder())
        .horizontal(|mut strip| {
            strip.cell(|ui| {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    if ui.button("Add Layer").clicked() {
                        args.gui_local.new_layer_modal_open = true;
                    }
                    if ui.button("Rearrange").clicked() {
                        args.gui_local.rearrange_layers_modal.new_order =
                            args.config.profiles[profile_idx].layers.clone();
                        args.gui_local.rearrange_layers_modal.modal_open = true;
                    }
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.label(&args.config.profiles[profile_idx].condition.helper_text());
                        let edit_response = ui.button("Edit");
                        if edit_response.clicked() {
                            args.gui_local.edit_profile_modal = EditProfileModalOpts {
                                modal_open: true,
                                name: args.config.profiles[profile_idx].name.clone(),
                                condition: match &args.config.profiles[profile_idx].condition {
                                    // custom
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
                                title: match &args.config.profiles[profile_idx].condition {
                                    config::ProfileCondition::TitleAndProcess {
                                        title,
                                        process: _,
                                    } => title.clone(),
                                    config::ProfileCondition::Title { title } => title.clone(),
                                    _ => String::new(),
                                },
                                process: match &args.config.profiles[profile_idx].condition {
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
                                let layer_select = ui_enable_clickable_table(
                                    ui,
                                    &mut args.config.profiles[profile_idx].layers,
                                    "Layer",
                                );
                                if let Some(i) = layer_select {
                                    args.gui_local.menu = GuiMenu::ProfileLayer {
                                        profile_idx,
                                        layer_idx: i,
                                    }
                                }
                            });
                    });
                });
            });
            strip.cell(|ui| {
                ui_base_layer(
                    ui,
                    &mut args.config.profiles[profile_idx].base,
                    &mut args.gui_local.new_base_remap_modal,
                );
            });
        });
    if args.gui_local.rearrange_layers_modal.modal_open {
        ui_rearrange_layers_modal(
            ui,
            &mut args.gui_local.rearrange_layers_modal,
            &mut args.config.profiles[profile_idx].layers,
        );
    }
    if args.gui_local.edit_profile_modal.modal_open {
        edit_profile_modal(
            ui,
            &mut args.gui_local.edit_profile_modal,
            &mut args.config.profiles[profile_idx],
        );
    }
    if args.gui_local.new_layer_modal_open {
        new_layer_modal(ui, args, profile_idx);
    }
}

fn edit_profile_modal(
    ui: &mut egui::Ui,
    modal_opts: &mut EditProfileModalOpts,
    profile: &mut config::Profile,
) {
    let ok_cancel =
        ui_edit_profile_modal(ui, modal_opts, &format!("Editing profile {}", profile.name));
    match ok_cancel {
        Some(true) => {
            profile.name = modal_opts.clone().name;
            profile.condition = modal_opts.clone().extract_condition();
            modal_opts.modal_open = false;
        }
        Some(false) => {
            modal_opts.modal_open = false;
        }
        None => (),
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

fn new_layer_modal(ui: &mut egui::Ui, args: &mut ReemApp, profile_idx: usize) {
    let ok_cancel = ui_ok_cancel_modal(ui, |ui| {
        ui.label("Layer Name");
        ui.text_edit_singleline(&mut args.gui_local.new_layer.name);
    });
    match ok_cancel {
        Some(true) => {
            let new_layer = args.gui_local.new_layer.clone();
            args.config.profiles[profile_idx].layers.push(new_layer);
            args.gui_local.new_layer_modal_open = false;
        }
        Some(false) => {
            args.gui_local.new_layer_modal_open = false;
        }
        None => (),
    }
}
