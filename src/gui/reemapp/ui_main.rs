use super::GuiMenu;
use super::ReemApp;

pub fn ui_main(ctx: &egui::Context, ui: &mut egui::Ui, args: &mut ReemApp) {
    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        if ui.button("Add Profile").clicked() {
            args.gui_local.new_profile_modal_open = true;
        }
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.heading("Profiles");
            });
            ui.add_space(super::SPACING);
            egui::Frame::new()
                .stroke(egui::Stroke {
                    width: 1.0,
                    color: egui::Color32::DARK_GRAY,
                })
                .inner_margin(4.0)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    profiles_table_ui(ui, args);
                });
        });
    });
    if args.gui_local.new_profile_modal_open {
        new_profile_modal(ctx, ui, args);
    }
}

fn profiles_table_ui(ui: &mut egui::Ui, args: &mut ReemApp) {
    enum ProfileSelect {
        None,
        Default,
        Other(usize),
    }
    use egui_extras::{Column, TableBuilder};
    let header_height = 12.0;
    let row_height = 20.0;
    let btn_size = [20.0, 20.0];
    let mut pointing_hand = false;
    let mut to_delete = None;
    let mut profile_select = ProfileSelect::None;
    TableBuilder::new(ui)
        .id_salt("Profiles Table")
        .striped(true)
        .auto_shrink(false)
        .sense(egui::Sense::click_and_drag())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(60.0)) // Enabled
        .column(Column::exact(60.0)) // Delete
        .column(Column::remainder()) // Profile Name
        // .column(Column::exact(60.0))
        .header(header_height, |mut header| {
            header.col(|ui| {
                ui.strong("Enabled");
            });
            header.col(|ui| {
                ui.strong("Remove");
            });
            header.col(|ui| {
                ui.strong("Name");
            });
        })
        .body(|mut body| {
            body.row(row_height, |mut row| {
                let mut dummy = true;
                row.col(|ui| {
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            ui.add_enabled(false, egui::Checkbox::without_text(&mut dummy));
                        },
                    );
                });
                row.col(|ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.add_enabled_ui(false, |ui| {
                            ui.add_sized(btn_size, egui::Button::new("✖"));
                        });
                    });
                });
                row.col(|ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    ui.label("Default Profile");
                });
                // row.col(|ui| {
                //     ui.add_enabled_ui(false, |ui| {
                //         ui.add_sized(btn_size, egui::Button::new("⬆"));
                //     });
                //     ui.add_enabled_ui(false, |ui| {
                //         ui.add_sized(btn_size, egui::Button::new("⬇"));
                //     });
                // });
                if row.response().hovered() {
                    pointing_hand = true;
                }
                if row.response().clicked() {
                    profile_select = ProfileSelect::Default;
                }
            });
            // let profiles_len = args.config.profiles.len();
            // let mut to_swap: Option<(usize, usize)> = None;
            for (i, profile) in args.config.profiles.iter_mut().enumerate() {
                // let first = i == 0;
                // let last = i == profiles_len - 1;
                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.with_layout(
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                ui.add(egui::Checkbox::without_text(&mut profile.enabled));
                            },
                        );
                    });
                    row.col(|ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            if ui.add_sized(btn_size, egui::Button::new("✖")).clicked() {
                                to_delete = Some(i);
                            };
                        });
                    });
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.label(&profile.name);
                    });
                    // row.col(|ui| {
                    //     ui.add_enabled_ui(!first, |ui| {
                    //         if ui.add_sized(btn_size, egui::Button::new("⬆")).clicked() {
                    //             to_swap = Some((i - 1, i));
                    //         }
                    //     });
                    //     ui.add_enabled_ui(!last, |ui| {
                    //         if ui.add_sized(btn_size, egui::Button::new("⬇")).clicked() {
                    //             to_swap = Some((i + 1, i));
                    //         }
                    //     });
                    // });
                    if row.response().hovered() {
                        pointing_hand = true;
                    }
                    if row.response().clicked() {
                        profile_select = ProfileSelect::Other(i);
                    }
                });
            }
            // if let Some((a, b)) = to_swap {
            //     args.config.profiles.swap(a, b);
            // }
        });
    if let Some(to_delete) = to_delete {
        args.config.profiles.remove(to_delete);
    }
    match profile_select {
        ProfileSelect::None => (),
        ProfileSelect::Default => {
            args.gui_local.menu = GuiMenu::DefaultProfileMenu;
        }
        ProfileSelect::Other(i) => args.gui_local.menu = GuiMenu::ProfileMenu { profile_idx: i },
    }
    if pointing_hand {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
}

fn new_profile_modal(ctx: &egui::Context, _ui: &mut egui::Ui, args: &mut ReemApp) {
    let mut ok = false;
    let mut cancel = false;
    let modal = egui::Modal::new(egui::Id::new("New Profile Modal")).show(ctx, |ui| {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            ui.label("Profile Name");
            ui.text_edit_singleline(&mut args.gui_local.new_profile.name);
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
        args.config
            .profiles
            .push(args.gui_local.new_profile.clone());
        args.gui_local.new_profile_modal_open = false;
    } else if cancel {
        args.gui_local.new_profile_modal_open = false;
    }
}
