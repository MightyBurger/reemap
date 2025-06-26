use crate::buttons;
use crate::config;
use crate::gui::reemapp::ui_remap_tables::{ui_available_remaps_table, ui_single_remap_table};
use crate::gui::reemapp::{NewRemapModalOpts, RemapPolicyUI};
use crate::gui::settings::LayerUI;
use strum::IntoEnumIterator;

pub fn ui_layer(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    layer: &mut LayerUI,
    new_remap_modal: &mut NewRemapModalOpts,
) {
    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        // TODO: if nothing needs to be at the bottom, remove the bottom_up layout.
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.label("Layer Name");
            ui.text_edit_singleline(&mut layer.name);
            ui.add_space(super::SPACING);

            egui::Frame::new()
                .stroke(egui::Stroke {
                    width: 1.0,
                    color: egui::Color32::DARK_GRAY,
                })
                .inner_margin(4.0)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui_remaps_table(ui, layer, new_remap_modal);
                });
        });
    });
    if let Some(button) = new_remap_modal.modal_open {
        let policy = &mut layer.policy[button];
        ui_new_remap_modal(ctx, ui, new_remap_modal, button, policy);
    }
}

pub fn ui_remaps_table(
    ui: &mut egui::Ui,
    layer: &mut LayerUI,
    new_remap_modal: &mut NewRemapModalOpts,
) {
    use egui_extras::{Column, TableBuilder};
    let header_height = 12.0;
    let row_height = 20.0;
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
                        let policy = layer.policy[button].clone();
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
    if let Some(button) = button_select {
        new_remap_modal.modal_open = Some(button);
        new_remap_modal.policy = match layer.policy[button] {
            config::RemapPolicy::Defer => RemapPolicyUI::Defer,
            config::RemapPolicy::NoRemap => RemapPolicyUI::NoRemap,
            config::RemapPolicy::Remap(_) => RemapPolicyUI::Remap,
        };
        new_remap_modal.outputs = match layer.policy[button] {
            config::RemapPolicy::Defer | config::RemapPolicy::NoRemap => Vec::new(),
            config::RemapPolicy::Remap(ref output) => output.clone(),
        };
    }
    if pointing_hand {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
}

fn ui_new_remap_modal(
    ctx: &egui::Context,
    _ui: &mut egui::Ui,
    modal_opts: &mut NewRemapModalOpts,
    button: buttons::Button,
    policy: &mut config::RemapPolicy,
) {
    let mut ok = false;
    let mut cancel = false;
    let modal = egui::Modal::new(egui::Id::new("New Remap Modal")).show(ctx, |ui| {
        ui.heading(format!("Remaps for {button}"));
        ui.separator();
        ui.add_space(super::SPACING);
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            egui::ComboBox::from_label("Policy")
                .selected_text(format!("{}", &modal_opts.policy))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut modal_opts.policy, RemapPolicyUI::Defer, "Defer");
                    ui.selectable_value(&mut modal_opts.policy, RemapPolicyUI::NoRemap, "No Remap");
                    ui.selectable_value(&mut modal_opts.policy, RemapPolicyUI::Remap, "Remap");
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
                ui.add_space(super::SPACING);
                ui.separator();
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    let enable_tables = match modal_opts.policy {
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
                                    ui_single_remap_table(ui, &mut modal_opts.outputs);
                                });
                            egui::Frame::new()
                                .stroke(egui::Stroke {
                                    width: 1.0,
                                    color: egui::Color32::DARK_GRAY,
                                })
                                .inner_margin(4.0)
                                .corner_radius(4.0)
                                .show(col_2, |ui| {
                                    ui_available_remaps_table(ui, &mut modal_opts.outputs);
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
        *policy = match modal_opts.policy {
            RemapPolicyUI::Defer => config::RemapPolicy::Defer,
            RemapPolicyUI::NoRemap => config::RemapPolicy::NoRemap,
            RemapPolicyUI::Remap => config::RemapPolicy::Remap(modal_opts.outputs.clone()),
        };
        modal_opts.modal_open = None;
    } else if cancel {
        modal_opts.modal_open = None;
    }
}
