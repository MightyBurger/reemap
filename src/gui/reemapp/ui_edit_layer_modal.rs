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
    show_rare_keys: bool,
) -> Option<bool> {
    let helper_text = config::Layer::from(modal_opts.clone()).condition_helper_text();
    ui_ok_cancel_modal(ui, &helper_text, true, |ui| {
        ui.heading(heading);
        ui.separator();
        ui.add_space(super::SPACING);

        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            ui.label("Layer name");
            ui.text_edit_singleline(&mut modal_opts.name);
            ui.add_space(super::SPACING);
            ui.label(
                "Layers let you override a profile's remaps when you hold down or toggle a key. \
                Multiple layers can be active at the same time. \
                Choose when this layer should be active below.",
            );
            ui.add_space(super::SPACING);

            ui.label("Layer type");
            egui::ComboBox::from_id_salt("layer type")
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
                                ui_available_layer_conditions_table(
                                    ui,
                                    &mut modal_opts.condition,
                                    show_rare_keys,
                                );
                            });
                    });
                });
            });
        });
    })
}

fn ui_available_layer_conditions_table(
    ui: &mut egui::Ui,
    remaps: &mut Vec<buttons::HoldButton>,
    show_rare_keys: bool,
) {
    use super::HEADER_HEIGHT;
    use super::ROW_HEIGHT;
    use buttons::HoldButton;
    use buttons::key::KeyType;
    use egui_extras::{Column, TableBuilder};

    let mut button_select = None;
    let mut pointing_hand = false;
    TableBuilder::new(ui)
        .id_salt("Available Hold Remaps Table")
        .striped(true)
        .auto_shrink(false)
        .sense(egui::Sense::click_and_drag())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(60.0)) // Device
        .column(Column::remainder()) // Button Name
        .header(HEADER_HEIGHT, |mut header| {
            header.col(|ui| {
                ui.strong("Device");
            });
            header.col(|ui| {
                ui.strong("Button");
            });
        })
        .body(|mut body| {
            let key_iter = buttons::key::KeyButton::iter()
                .filter(|key| match (show_rare_keys, key.key_type()) {
                    (true, KeyType::Common | KeyType::Rare) => true,
                    (false, KeyType::Common) => true,
                    _ => false,
                })
                .map(buttons::HoldButton::from);
            let mouse_iter = buttons::mouse::MouseButton::iter().map(buttons::HoldButton::from);

            for button in mouse_iter.chain(key_iter) {
                let enabled = !remaps.contains(&button);
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        let device = match button {
                            HoldButton::Key(_) => "Keyboard",
                            HoldButton::Mouse(_) => "Mouse",
                        };
                        ui.add_enabled(enabled, egui::Label::new(format!("{device}")));
                    });
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
