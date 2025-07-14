use crate::buttons;
use crate::config;
use crate::gui::reemapp::ui_ok_cancel_modal::ui_ok_cancel_modal;
use crate::gui::reemapp::ui_tables::ui_available_remaps_table;
use crate::gui::reemapp::ui_tables::ui_rearrange_table;
use crate::gui::reemapp::{BaseRemapPolicyUI, NewBaseRemapModalOpts};
use smallvec::SmallVec;
use strum::IntoEnumIterator;

pub fn ui_base_layer(
    ui: &mut egui::Ui,
    layer: &mut config::BaseLayer,
    new_remap_modal: &mut NewBaseRemapModalOpts,
) {
    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
        egui::Frame::new()
            .stroke(egui::Stroke {
                width: 1.0,
                color: egui::Color32::DARK_GRAY,
            })
            .inner_margin(4.0)
            .corner_radius(4.0)
            .show(ui, |ui| {
                ui_base_remaps_table(ui, layer, new_remap_modal);
            });
    });
    if let Some(button) = new_remap_modal.modal_open {
        let policy = &mut layer.policy[button];
        ui_new_base_remap_modal(ui, new_remap_modal, button, policy);
    }
}

pub fn ui_base_remaps_table(
    ui: &mut egui::Ui,
    layer: &mut config::BaseLayer,
    new_remap_modal: &mut NewBaseRemapModalOpts,
) {
    use egui_extras::{Column, TableBuilder};
    let header_height = 12.0;
    let row_height = 20.0;
    let mut pointing_hand = false;
    let mut button_select = None;
    TableBuilder::new(ui)
        .id_salt("Base Remaps Table")
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
            config::BaseRemapPolicy::NoRemap => BaseRemapPolicyUI::NoRemap,
            config::BaseRemapPolicy::Remap(_) => BaseRemapPolicyUI::Remap,
        };
        new_remap_modal.outputs = match layer.policy[button] {
            config::BaseRemapPolicy::NoRemap => SmallVec::new(),
            config::BaseRemapPolicy::Remap(ref output) => output.clone(),
        };
    }
    if pointing_hand {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
}

fn ui_new_base_remap_modal(
    ui: &mut egui::Ui,
    modal_opts: &mut NewBaseRemapModalOpts,
    button: buttons::Button,
    policy: &mut config::BaseRemapPolicy,
) {
    let ok_cancel = ui_ok_cancel_modal(ui, |ui| {
        ui.heading(format!("Remaps for {button}"));
        ui.separator();
        ui.add_space(super::SPACING);
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            egui::ComboBox::from_label("Policy")
                .selected_text(format!("{}", &modal_opts.policy))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut modal_opts.policy,
                        BaseRemapPolicyUI::NoRemap,
                        "No Remap",
                    );
                    ui.selectable_value(&mut modal_opts.policy, BaseRemapPolicyUI::Remap, "Remap");
                });

            ui.add_space(super::SPACING);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.label(get_new_remap_helper_text_base(
                    &button,
                    &modal_opts.outputs,
                    &modal_opts.policy,
                ));
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    let enable_tables = match modal_opts.policy {
                        BaseRemapPolicyUI::NoRemap => false,
                        BaseRemapPolicyUI::Remap => true,
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
                BaseRemapPolicyUI::NoRemap => config::BaseRemapPolicy::NoRemap,
                BaseRemapPolicyUI::Remap => {
                    config::BaseRemapPolicy::Remap(modal_opts.outputs.clone())
                }
            };
            modal_opts.modal_open = None;
        }
        Some(false) => {
            modal_opts.modal_open = None;
        }
        None => (),
    }
}

fn get_new_remap_helper_text_base(
    button: &buttons::Button,
    outputs: &[buttons::Button],
    policy: &BaseRemapPolicyUI,
) -> String {
    match policy {
        BaseRemapPolicyUI::NoRemap => {
            format!("{button} will not be remapped.")
        }
        BaseRemapPolicyUI::Remap => {
            let buttons_str: String = if outputs.is_empty() {
                String::from("(no inputs)")
            } else {
                itertools::Itertools::intersperse(
                    outputs.iter().map(|btn| btn.to_string()),
                    String::from(", "),
                )
                .collect()
            };
            format!("{button} will be remapped to the following: {buttons_str}.")
        }
    }
}
