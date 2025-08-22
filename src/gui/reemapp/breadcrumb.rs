// Copyright 2025 Jordan Johnson
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! UI for the buttons across the top of the window.

use super::GuiMenu;
use super::ReemApp;
use crate::gui::reemapp::style;

pub fn breadcrumb(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    args: &mut ReemApp,
    unsaved_changes: bool,
) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
        ui.add_enabled_ui(unsaved_changes, |ui| {
            if ui
                .add_sized(style::BUTTON_SIZE, egui::Button::new("Cancel"))
                .clicked()
            {
                args.discard_changes();
            }
            if ui
                .add_sized(style::BUTTON_SIZE, egui::Button::new("Apply"))
                .clicked()
            {
                args.apply_changes();
            }
        });
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            if let Some(click) = breadcrumb_display(ctx, ui, args) {
                args.gui_local.menu = click;
            }
        });
    });
}

fn breadcrumb_display(_ctx: &egui::Context, ui: &mut egui::Ui, args: &ReemApp) -> Option<GuiMenu> {
    let mut click = None;

    // -------------------- Main Menu Button --------------------

    let main_breadcrumb_response =
        ui.add(egui::Label::new(egui::RichText::new("Home").heading()).sense(egui::Sense::click()));
    if main_breadcrumb_response.hovered() {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
    if main_breadcrumb_response.clicked() {
        click = Some(GuiMenu::Main);
    }
    if args.gui_local.menu == GuiMenu::Main {
        return click;
    }

    // -------------------- Profile Button --------------------

    let profile_idx = match &args.gui_local.menu {
        GuiMenu::Main => unreachable!(),
        GuiMenu::Profile { profile_idx } => *profile_idx,
        GuiMenu::ProfileLayer {
            profile_idx,
            layer_idx: _,
        } => *profile_idx,
    };
    let profile_string = &args.config.profiles[profile_idx].name;
    ui.heading(" > ");
    let profile_breadcrumb_response = ui.add(
        egui::Label::new(egui::RichText::new(profile_string).heading())
            .truncate()
            .sense(egui::Sense::click()),
    );
    if profile_breadcrumb_response.hovered() {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
    if profile_breadcrumb_response.clicked() {
        click = Some(GuiMenu::Profile { profile_idx });
    }
    if matches!(args.gui_local.menu, GuiMenu::Main | GuiMenu::Profile { .. }) {
        return click;
    }

    // -------------------- Layer Button --------------------

    let layer_idx = match &args.gui_local.menu {
        GuiMenu::Main | GuiMenu::Profile { .. } => {
            unreachable!()
        }
        GuiMenu::ProfileLayer {
            profile_idx: _,
            layer_idx,
        } => *layer_idx,
    };

    let layer_string = &args.config.profiles[profile_idx].layers[layer_idx].name;

    ui.heading(" > ");

    let layer_breadcrumb_response = ui.add(
        egui::Label::new(egui::RichText::new(layer_string).heading())
            .truncate()
            .sense(egui::Sense::click()),
    );
    if layer_breadcrumb_response.hovered() {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
    if layer_breadcrumb_response.clicked() {
        click = Some(GuiMenu::ProfileLayer {
            profile_idx,
            layer_idx,
        });
    }
    click
}
