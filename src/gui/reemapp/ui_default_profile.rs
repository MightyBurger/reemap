use crate::gui::reemapp::ui_base_layer::ui_base_layer;
use crate::gui::reemapp::ui_tables::ui_enable_clickable_table;

use super::GuiMenu;
use super::ReemApp;
use egui_extras::{Size, StripBuilder};

pub fn ui_default_profile(ctx: &egui::Context, ui: &mut egui::Ui, args: &mut ReemApp) {
    StripBuilder::new(ui)
        .size(Size::relative(0.5))
        .size(Size::remainder())
        .horizontal(|mut strip| {
            strip.cell(|ui| {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    if ui.button("Add Layer").clicked() {
                        args.gui_local.new_default_layer_modal_open = true;
                    }
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.label("Active when no other profile is active");
                        ui.add_space(super::SPACING);

                        egui::Frame::new()
                            .stroke(egui::Stroke {
                                width: 1.0,
                                color: egui::Color32::DARK_GRAY,
                            })
                            .inner_margin(4.0)
                            .corner_radius(4.0)
                            .show(ui, |ui| {
                                let layer_select = ui_enable_clickable_table(
                                    ui,
                                    &mut args.config.default.layers,
                                    "Layer",
                                );
                                if let Some(i) = layer_select {
                                    args.gui_local.menu =
                                        GuiMenu::DefaultProfileLayer { layer_idx: i };
                                }
                            });
                    });
                });
            });
            strip.cell(|ui| {
                ui_base_layer(
                    ctx,
                    ui,
                    &mut args.config.default.base,
                    &mut args.gui_local.new_base_remap_modal,
                );
            });
        });
    if args.gui_local.new_default_layer_modal_open {
        new_default_layer_modal(ctx, ui, args);
    }
}

fn new_default_layer_modal(ctx: &egui::Context, _ui: &mut egui::Ui, args: &mut ReemApp) {
    let mut ok = false;
    let mut cancel = false;
    let modal = egui::Modal::new(egui::Id::new("New Layer Modal")).show(ctx, |ui| {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            ui.label("Layer Name");
            ui.text_edit_singleline(&mut args.gui_local.new_layer.name);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Cancel").clicked() {
                    cancel = true;
                }
                if ui.button("OK").clicked() {
                    ok = true;
                }
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
        let new_layer = args.gui_local.new_layer.clone();
        args.config.default.layers.push(new_layer);
        args.gui_local.new_default_layer_modal_open = false;
    } else if cancel {
        args.gui_local.new_default_layer_modal_open = false;
    }
}
