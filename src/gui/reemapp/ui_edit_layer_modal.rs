use crate::buttons;
use crate::config;
use crate::gui::reemapp::EditLayerModalOpts;
use crate::gui::reemapp::ui_ok_cancel_modal::ui_ok_cancel_modal;
use crate::gui::reemapp::ui_tables::ui_rearrange_table;
use strum::IntoEnumIterator;

pub fn ui_edit_layer_modal(
    ui: &mut egui::Ui,
    modal_opts: &mut EditLayerModalOpts,
    heading: &str,
) -> Option<bool> {
    ui_ok_cancel_modal(ui, |ui| {
        ui.heading(heading);
        ui.separator();
        ui.add_space(super::SPACING);

        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            ui.label("Layer name");
            ui.text_edit_singleline(&mut modal_opts.name);
            ui.separator();

            egui::ComboBox::from_label("Layer type")
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
                ui.label(config::Layer::from(modal_opts.clone()).condition_helper_text());
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
                                ui_rearrange_table(
                                    ui,
                                    &mut modal_opts.condition,
                                    "Layer conditions",
                                );
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
    })
}

fn ui_available_layer_conditions_table(ui: &mut egui::Ui, remaps: &mut Vec<buttons::HoldButton>) {
    use egui_extras::{Column, TableBuilder};
    let header_height = 12.0;
    let row_height = 20.0;
    let mut button_select = None;
    let mut pointing_hand = false;
    TableBuilder::new(ui)
        .id_salt("Available Hold Remaps Table")
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
            let key_iter = buttons::key::KeyButton::iter().map(buttons::HoldButton::from);
            let mouse_iter = buttons::mouse::MouseButton::iter().map(buttons::HoldButton::from);

            for button in key_iter.chain(mouse_iter) {
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
