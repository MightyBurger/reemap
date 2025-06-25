use super::ReemApp;
use crate::config;
use crate::{buttons, gui::reemapp::RemapPolicyUI};
use strum::IntoEnumIterator;

pub fn ui_profile_layer(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    args: &mut ReemApp,
    profile_idx: usize,
    layer_idx: usize,
) {
    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        // TODO: if nothing needs to be at the bottom, remove the bottom_up layout.
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.label("Layer Name");
            ui.text_edit_singleline(&mut args.config.profiles[profile_idx].layers[layer_idx].name);
            ui.add_space(super::SPACING);

            egui::Frame::new()
                .stroke(egui::Stroke {
                    width: 1.0,
                    color: egui::Color32::DARK_GRAY,
                })
                .inner_margin(4.0)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    remaps_table(ui, args, profile_idx, layer_idx);
                });
        });
    });
    if let Some(button) = args.gui_local.new_remap_modal_open {
        new_remap_modal(ctx, ui, args, profile_idx, layer_idx, button);
    }
}

fn remaps_table(ui: &mut egui::Ui, args: &mut ReemApp, profile_idx: usize, layer_idx: usize) {
    use egui_extras::{Column, TableBuilder};
    let header_height = 12.0;
    let row_height = 20.0;
    let btn_size = [20.0, 20.0];
    let mut pointing_hand = false;
    let mut button_select = None;
    TableBuilder::new(ui)
        .id_salt("Remaps Table")
        .striped(true)
        .auto_shrink(false)
        .sense(egui::Sense::click_and_drag())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(120.0)) // Enabled
        .column(Column::remainder()) // Profile Name
        .header(header_height, |mut header| {
            header.col(|ui| {
                ui.strong("Input");
            });
            header.col(|ui| {
                ui.strong("Output");
            });
        })
        .body(|mut body| {
            let key_iter = buttons::key::KeyButton::iter().map(|key| buttons::Button::from(key));
            let mouse_iter =
                buttons::mouse::MouseButton::iter().map(|mouse| buttons::Button::from(mouse));
            let wheel_iter =
                buttons::wheel::MouseWheelButton::iter().map(|wheel| buttons::Button::from(wheel));

            for button in key_iter.chain(mouse_iter).chain(wheel_iter) {
                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.label(format!("{button}"));
                    });
                    row.col(|ui| {
                        let policy = args.config.profiles[profile_idx].layers[layer_idx].policy
                            [button]
                            .clone();
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.label(format!("{policy}"));
                    });
                    if row.response().hovered() {
                        pointing_hand = true;
                    }
                    if row.response().clicked() {
                        button_select = Some(button);
                    }
                });
            }
        });
    if matches!(button_select, Some(_)) {
        args.gui_local.new_remap_modal_open = button_select;
    }
    if pointing_hand {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
}

fn single_remap_table(ui: &mut egui::Ui, remaps: &mut Vec<buttons::Button>) {
    use egui_extras::{Column, TableBuilder};
    let header_height = 12.0;
    let row_height = 20.0;
    let btn_size = [20.0, 20.0];
    let mut to_delete = None;
    let mut pointing_hand = false;
    TableBuilder::new(ui)
        .id_salt("Single Remap Table")
        .striped(true)
        .auto_shrink(false)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(60.0)) // Enabled
        .column(Column::remainder()) // Profile Name
        .header(header_height, |mut header| {
            header.col(|ui| {
                ui.strong("Remove");
            });
            header.col(|ui| {
                ui.strong("Output");
            });
        })
        .body(|mut body| {
            for (i, button) in remaps.iter_mut().enumerate() {
                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            let remove_btn_response =
                                ui.add_sized(btn_size, egui::Button::new("✖"));
                            if remove_btn_response.hovered() {
                                pointing_hand = true;
                            }
                            if remove_btn_response.clicked() {
                                to_delete = Some(i);
                            };
                        });
                    });
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.label(format!("{button}"));
                    });
                });
            }
        });
    if let Some(to_delete) = to_delete {
        remaps.remove(to_delete);
    }
    if pointing_hand {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
}

fn available_remaps_table(ui: &mut egui::Ui, remaps: &mut Vec<buttons::Button>) {
    use egui_extras::{Column, TableBuilder};
    let header_height = 12.0;
    let row_height = 20.0;
    let mut button_select = None;
    let mut pointing_hand = false;
    TableBuilder::new(ui)
        .id_salt("Available Remaps Table")
        .striped(true)
        .auto_shrink(false)
        .sense(egui::Sense::click_and_drag())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder()) // Button Name
        .header(header_height, |mut header| {
            header.col(|ui| {
                ui.strong("Button");
            });
        })
        .body(|mut body| {
            let key_iter = buttons::key::KeyButton::iter().map(|key| buttons::Button::from(key));
            let mouse_iter =
                buttons::mouse::MouseButton::iter().map(|mouse| buttons::Button::from(mouse));
            let wheel_iter =
                buttons::wheel::MouseWheelButton::iter().map(|wheel| buttons::Button::from(wheel));

            for button in key_iter.chain(mouse_iter).chain(wheel_iter) {
                let enabled = !remaps.contains(&button);
                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.add_enabled(enabled, egui::Label::new(format!("{button}")));
                    });
                    if enabled && row.response().hovered() {
                        pointing_hand = true;
                    }
                    if enabled && row.response().clicked() {
                        button_select = Some(button);
                    }
                });
            }
        });
    if pointing_hand {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
    if let Some(button_select) = button_select {
        if !remaps.contains(&button_select) {
            remaps.push(button_select);
        }
    }
}

fn new_remap_modal(
    ctx: &egui::Context,
    _ui: &mut egui::Ui,
    args: &mut ReemApp,
    profile_idx: usize,
    layer_idx: usize,
    button: buttons::Button,
) {
    let mut ok = false;
    let mut cancel = false;
    let modal = egui::Modal::new(egui::Id::new("New Remap Modal")).show(ctx, |ui| {
        ui.heading(format!("Remaps for {button}"));
        ui.separator();
        ui.add_space(super::SPACING);
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            egui::ComboBox::from_label("Policy")
                .selected_text(format!("{}", &args.gui_local.new_remap_policy))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut args.gui_local.new_remap_policy,
                        RemapPolicyUI::Defer,
                        "Defer",
                    );
                    ui.selectable_value(
                        &mut args.gui_local.new_remap_policy,
                        RemapPolicyUI::NoRemap,
                        "No Remap",
                    );
                    ui.selectable_value(
                        &mut args.gui_local.new_remap_policy,
                        RemapPolicyUI::Remap,
                        "Remap",
                    );
                });

            ui.add_space(super::SPACING);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                    if ui.button("OK").clicked() {
                        ok = true;
                    }
                });
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    let enable_tables = match args.gui_local.new_remap_policy {
                        RemapPolicyUI::Defer | RemapPolicyUI::NoRemap => false,
                        RemapPolicyUI::Remap => true,
                    };
                    ui.add_enabled_ui(enable_tables, |ui| {
                        ui.columns_const(|[col_1, col_2]| {
                            egui::Frame::new()
                                .stroke(egui::Stroke {
                                    width: 1.0,
                                    color: egui::Color32::DARK_GRAY,
                                })
                                .inner_margin(4.0)
                                .corner_radius(4.0)
                                .show(col_1, |ui| {
                                    single_remap_table(ui, &mut args.gui_local.new_remap_outputs);
                                });
                            egui::Frame::new()
                                .stroke(egui::Stroke {
                                    width: 1.0,
                                    color: egui::Color32::DARK_GRAY,
                                })
                                .inner_margin(4.0)
                                .corner_radius(4.0)
                                .show(col_2, |ui| {
                                    available_remaps_table(
                                        ui,
                                        &mut args.gui_local.new_remap_outputs,
                                    );
                                });
                        });
                    });
                });
            });
        });
    });
    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
        ok = true;
    }
    if modal.should_close() {
        cancel = true;
    }
    if ok {
        args.config.profiles[profile_idx].layers[layer_idx].policy[button] =
            match args.gui_local.new_remap_policy {
                RemapPolicyUI::Defer => config::RemapPolicy::Defer,
                RemapPolicyUI::NoRemap => config::RemapPolicy::NoRemap,
                RemapPolicyUI::Remap => {
                    config::RemapPolicy::Remap(args.gui_local.new_remap_outputs.clone())
                }
            };
        println!("success");
        args.gui_local.new_remap_modal_open = None;
    } else if cancel {
        args.gui_local.new_remap_modal_open = None;
    }
}
