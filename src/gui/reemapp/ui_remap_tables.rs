use crate::buttons;
use strum::IntoEnumIterator;

pub fn ui_single_remap_table(ui: &mut egui::Ui, remaps: &mut Vec<buttons::Button>) {
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

pub fn ui_available_remaps_table(ui: &mut egui::Ui, remaps: &mut Vec<buttons::Button>) {
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
