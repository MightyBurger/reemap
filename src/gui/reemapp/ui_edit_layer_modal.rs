// Copyright 2025 Jordan Johnson
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The modal for when the user creates or edits a layer.

use crate::config;
use crate::gui::reemapp::EditLayerModalOpts;
use crate::gui::reemapp::style;
use crate::gui::reemapp::ui_ok_cancel_modal::ui_ok_cancel_modal;
use crate::gui::reemapp::ui_tables::ui_available_hold_buttons_table;
use crate::gui::reemapp::ui_tables::ui_rearrange_table;

pub fn ui_edit_layer_modal(
    ui: &mut egui::Ui,
    modal_opts: &mut EditLayerModalOpts,
    heading: &str,
    show_rare_keys: bool,
) -> Option<bool> {
    use egui_extras::{Size, StripBuilder};

    let valid = !modal_opts.name.is_empty() && !modal_opts.condition.is_empty();
    let helper_text = if valid {
        config::Layer::from(modal_opts.clone()).condition_helper_text()
    } else if modal_opts.name.is_empty() {
        String::from("Choose a layer name")
    } else {
        String::from("Choose one or more inputs")
    };

    ui_ok_cancel_modal(ui, &helper_text, valid, |ui| {
        ui.heading(heading);
        ui.separator();
        ui.add_space(style::SPACING);

        ui.label(
            "Layers let you override a profile's remaps when you hold down or toggle a key. \
                Multiple layers can be active at the same time.",
        );
        ui.add_space(style::SPACING);

        egui::Grid::new("edit_layer_modal_grid1")
            .num_columns(2)
            .spacing([style::SPACING, style::SPACING])
            .show(ui, |ui| {
                ui.label("Layer name");
                ui.add(
                    egui::TextEdit::singleline(&mut modal_opts.name).hint_text("Insert layer name"),
                );
                ui.end_row();
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
                ui.end_row();
            });
        ui.add_space(style::SPACING);

        ui.columns_const(|[col_1, col_2]| {
            style::UI_FRAME.show(col_1, |ui| {
                ui_rearrange_table(ui, &mut modal_opts.condition, "Layer conditions");
            });
            StripBuilder::new(col_2)
                .size(Size::remainder())
                .size(Size::initial(style::BUTTON_HEIGHT))
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        style::UI_FRAME.show(ui, |ui| {
                            ui_available_hold_buttons_table(
                                ui,
                                &mut modal_opts.condition,
                                &modal_opts.search,
                                show_rare_keys,
                            );
                        });
                    });
                    strip.cell(|ui| {
                        ui.add_sized(
                            [ui.available_width(), style::BUTTON_HEIGHT],
                            egui::TextEdit::singleline(&mut modal_opts.search).hint_text("Search"),
                        );
                    });
                });
        });
    })
}
