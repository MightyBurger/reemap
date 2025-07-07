use crate::gui::reemapp::ui_base_layer::ui_base_layer;

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
                                default_layers_table_ui(ui, args);
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
fn default_layers_table_ui(ui: &mut egui::Ui, args: &mut ReemApp) {
    use egui_extras::{Column, TableBuilder};
    let header_height = 12.0;
    let row_height = 20.0;
    let btn_size = [20.0, 20.0];
    let mut pointing_hand = false;
    let mut to_delete = None;
    let mut layer_select = None;
    let layers_len = args.config.default.layers.len();
    let mut to_swap: Option<(usize, usize)> = None;
    TableBuilder::new(ui)
        .id_salt("Default Layers Table")
        .striped(true)
        .auto_shrink(false)
        .sense(egui::Sense::click_and_drag())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(30.0)) // Enabled
        .column(Column::remainder()) // Profile Name
        .column(Column::exact(70.0))
        .header(header_height, |mut header| {
            header.col(|_ui| {
                // ui.strong("Enabled");
            });
            header.col(|ui| {
                ui.strong("Layer");
            });
            header.col(|ui| {
                ui.strong("Move");
            });
        })
        .body(|mut body| {
            for (i, layer) in args.config.default.layers.iter_mut().enumerate() {
                let first = i == 0;
                let last = i == layers_len - 1;
                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.with_layout(
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                ui.add(egui::Checkbox::without_text(&mut layer.enabled));
                            },
                        );
                    });
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.label(&layer.name);
                    });
                    row.col(|ui| {
                        ui.style_mut().spacing.item_spacing = [2.0, 2.0].into();
                        ui.add_enabled_ui(!first, |ui| {
                            if ui.add_sized(btn_size, egui::Button::new("⬆")).clicked() {
                                to_swap = Some((i - 1, i));
                            }
                        });
                        ui.add_enabled_ui(!last, |ui| {
                            if ui.add_sized(btn_size, egui::Button::new("⬇")).clicked() {
                                to_swap = Some((i + 1, i));
                            }
                        });
                        if ui.add_sized(btn_size, egui::Button::new("✖")).clicked() {
                            to_delete = Some(i);
                        };
                    });
                    if row.response().hovered() {
                        pointing_hand = true;
                    }
                    if row.response().clicked() {
                        layer_select = Some(i);
                    }
                });
            }
        });
    if let Some((a, b)) = to_swap {
        args.config.default.layers.swap(a, b);
    }
    if let Some(to_delete) = to_delete {
        args.config.default.layers.remove(to_delete);
    }
    if let Some(i) = layer_select {
        args.gui_local.menu = GuiMenu::DefaultProfileLayer { layer_idx: i };
    }
    if pointing_hand {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
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
