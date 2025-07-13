use crate::buttons;
use crate::config;
use crate::gui::reemapp::ui_ok_cancel_modal::ui_ok_cancel_modal;
use crate::gui::reemapp::ui_tables::ui_rearrange_table;
use crate::gui::reemapp::ui_tables::{
    ui_available_layer_conditions_table, ui_available_remaps_table, ui_layer_condition_table,
};
use crate::gui::reemapp::{LayerConditionModalOpts, NewRemapModalOpts, RemapPolicyUI};
use smallvec::SmallVec;
use strum::IntoEnumIterator;

pub fn ui_layer(
    ui: &mut egui::Ui,
    layer: &mut config::Layer,
    new_remap_modal: &mut NewRemapModalOpts,
    layer_condition_modal: &mut LayerConditionModalOpts,
) {
    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
        ui.label("Layer Name");
        ui.text_edit_singleline(&mut layer.name);
        ui.add_space(super::SPACING);

        ui.label(get_layer_condition_text(
            &layer.condition,
            &layer.layer_type,
        ));
        let edit_response = ui.button("Edit condition");
        if edit_response.clicked() {
            *layer_condition_modal = LayerConditionModalOpts {
                modal_open: true,
                layer_type: layer.layer_type.clone(),
                condition: layer.condition.clone(),
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
                ui_remaps_table(ui, layer, new_remap_modal);
            });
    });
    if let Some(button) = new_remap_modal.modal_open {
        let policy = &mut layer.policy[button];
        ui_new_remap_modal(ui, new_remap_modal, button, policy);
    }
    if layer_condition_modal.modal_open {
        let layer_name = &layer.name;
        let layer_type = &mut layer.layer_type;
        let condition = &mut layer.condition;
        ui_layer_condition_modal(ui, layer_condition_modal, layer_name, layer_type, condition);
    }
}

pub fn ui_remaps_table(
    ui: &mut egui::Ui,
    layer: &mut config::Layer,
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
            let key_iter = buttons::key::KeyButton::iter().map(buttons::Button::from);
            let mouse_iter = buttons::mouse::MouseButton::iter().map(buttons::Button::from);
            let wheel_iter = buttons::wheel::MouseWheelButton::iter().map(buttons::Button::from);

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
            config::RemapPolicy::Defer | config::RemapPolicy::NoRemap => SmallVec::new(),
            config::RemapPolicy::Remap(ref output) => output.clone(),
        };
    }
    if pointing_hand {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
}

fn ui_new_remap_modal(
    ui: &mut egui::Ui,
    modal_opts: &mut NewRemapModalOpts,
    button: buttons::Button,
    policy: &mut config::RemapPolicy,
) {
    let ok_cancel = ui_ok_cancel_modal(ui, |ui| {
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
                ui.label(get_new_remap_helper_text(
                    &button,
                    &modal_opts.outputs,
                    &modal_opts.policy,
                ));
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
                                    ui_rearrange_table(ui, &mut modal_opts.outputs, "Output");
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
    match ok_cancel {
        Some(true) => {
            *policy = match modal_opts.policy {
                RemapPolicyUI::Defer => config::RemapPolicy::Defer,
                RemapPolicyUI::NoRemap => config::RemapPolicy::NoRemap,
                RemapPolicyUI::Remap => config::RemapPolicy::Remap(modal_opts.outputs.clone()),
            };
            modal_opts.modal_open = None;
        }
        Some(false) => {
            modal_opts.modal_open = None;
        }
        None => (),
    }
}

fn ui_layer_condition_modal(
    ui: &mut egui::Ui,
    modal_opts: &mut LayerConditionModalOpts,
    layer_name: &str,
    layer_type: &mut config::LayerType,
    condition: &mut Vec<buttons::HoldButton>,
) {
    let ok_cancel = ui_ok_cancel_modal(ui, |ui| {
        ui.heading(format!("Condition for {layer_name}"));
        ui.separator();
        ui.add_space(super::SPACING);

        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            egui::ComboBox::from_label("Layer Type")
                .selected_text(format!("{}", &modal_opts.layer_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut modal_opts.layer_type,
                        config::LayerType::Modifier,
                        "Modifier",
                    );
                    ui.selectable_value(
                        &mut modal_opts.layer_type,
                        config::LayerType::Toggle,
                        "Toggle",
                    );
                });
            ui.add_space(super::SPACING);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.label(get_layer_condition_text(
                    &modal_opts.condition,
                    &modal_opts.layer_type,
                ));
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.columns_const(|[col_1, col_2]| {
                        egui::Frame::new()
                            .stroke(egui::Stroke {
                                width: 1.0,
                                color: egui::Color32::DARK_GRAY,
                            })
                            .inner_margin(4.0)
                            .corner_radius(4.0)
                            .show(col_1, |ui| {
                                ui_layer_condition_table(ui, &mut modal_opts.condition);
                            });
                        egui::Frame::new()
                            .stroke(egui::Stroke {
                                width: 1.0,
                                color: egui::Color32::DARK_GRAY,
                            })
                            .inner_margin(4.0)
                            .corner_radius(4.0)
                            .show(col_2, |ui| {
                                ui_available_layer_conditions_table(ui, &mut modal_opts.condition);
                            });
                    });
                });
            });
        });
    });
    match ok_cancel {
        Some(true) => {
            *layer_type = modal_opts.layer_type.clone();
            *condition = modal_opts.condition.clone();
            modal_opts.modal_open = false;
        }
        Some(false) => {
            modal_opts.modal_open = false;
        }
        None => (),
    }
}

fn get_layer_condition_text(
    condition: &[buttons::HoldButton],
    layer_type: &config::LayerType,
) -> String {
    let condition_buttons_str: String = if condition.is_empty() {
        String::from("(no buttons set)")
    } else {
        itertools::Itertools::intersperse(
            condition.iter().map(|btn| btn.to_string()),
            String::from(", "),
        )
        .collect()
    };
    match layer_type {
        config::LayerType::Modifier => {
            format!("Active when these buttons are held: {condition_buttons_str}")
        }
        config::LayerType::Toggle => {
            format!("Toggled when these buttons are pressed: {condition_buttons_str}")
        }
    }
}

fn get_new_remap_helper_text(
    button: &buttons::Button,
    outputs: &[buttons::Button],
    policy: &RemapPolicyUI,
) -> String {
    match policy {
        RemapPolicyUI::Defer => format!("This layer will not affect {button} inputs."),
        RemapPolicyUI::NoRemap => {
            format!("When active, this layer will prevent {button} from being remapped.")
        }
        RemapPolicyUI::Remap => {
            let buttons_str: String = if outputs.is_empty() {
                String::from("(no inputs)")
            } else {
                itertools::Itertools::intersperse(
                    outputs.iter().map(|btn| btn.to_string()),
                    String::from(", "),
                )
                .collect()
            };
            format!("When active, this layer will remap {button} to the following: {buttons_str}.")
        }
    }
}
