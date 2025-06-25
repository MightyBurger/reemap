use crate::config;
use crate::gui::reemapp::ui_new_remap_modal::ui_new_remap_modal;
use crate::gui::reemapp::{LayerUI, NewRemapModalOpts};
use crate::{buttons, gui::reemapp::RemapPolicyUI};
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
                    remaps_table(ui, layer, new_remap_modal);
                });
        });
    });
    if let Some(button) = new_remap_modal.modal_open {
        let policy = &mut layer.policy[button];
        ui_new_remap_modal(ctx, ui, new_remap_modal, button, policy);
    }
}

fn remaps_table(ui: &mut egui::Ui, layer: &mut LayerUI, new_remap_modal: &mut NewRemapModalOpts) {
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
