use super::GuiMenu;
use super::ReemApp;
use crate::config;
use crate::gui::reemapp::ProfileConditionModalOpts;
use crate::gui::reemapp::ProfileConditionUI;
use crate::query_windows;

use egui_extras::{Size, StripBuilder};

pub fn ui_profile(ctx: &egui::Context, ui: &mut egui::Ui, args: &mut ReemApp, profile_idx: usize) {
    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        if ui.button("Add Layer").clicked() {
            args.gui_local.new_layer_modal_open = true;
        }
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.label("Profile Name");
            ui.text_edit_singleline(&mut args.config.profiles[profile_idx].name);
            ui.add_space(super::SPACING);

            ui.label(get_profile_condition_text(
                &args.config.profiles[profile_idx].condition,
            ));
            let edit_response = ui.button("Edit condition");
            if edit_response.clicked() {
                args.gui_local.profile_condition_modal = ProfileConditionModalOpts {
                    modal_open: true,
                    condition: match &args.config.profiles[profile_idx].condition {
                        // custom
                        config::ProfileCondition::TitleAndProcess { .. } => {
                            ProfileConditionUI::TitleAndProcess
                        }
                        config::ProfileCondition::Title { .. } => ProfileConditionUI::Title,
                        config::ProfileCondition::Process { .. } => ProfileConditionUI::Process,
                        // presets
                        config::ProfileCondition::OriBF => ProfileConditionUI::OriBF,
                        config::ProfileCondition::OriBFDE => ProfileConditionUI::OriBFDE,
                        config::ProfileCondition::OriWotW => ProfileConditionUI::OriWotW,
                    },
                    title: match &args.config.profiles[profile_idx].condition {
                        config::ProfileCondition::TitleAndProcess { title, process: _ } => {
                            title.clone()
                        }
                        config::ProfileCondition::Title { title } => title.clone(),
                        _ => String::new(),
                    },
                    process: match &args.config.profiles[profile_idx].condition {
                        config::ProfileCondition::TitleAndProcess { title: _, process } => {
                            process.clone()
                        }
                        config::ProfileCondition::Process { process } => process.clone(),
                        _ => String::new(),
                    },
                    open_windows: query_windows::enumerate_open_windows(),
                };
            }
            ui.add_space(super::SPACING);

            egui::Frame::new()
                .stroke(egui::Stroke {
                    width: 1.0,
                    color: egui::Color32::DARK_GRAY,
                })
                .inner_margin(4.0)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    layers_table_ui(ui, args, profile_idx);
                });
        });
    });
    if args.gui_local.profile_condition_modal.modal_open {
        ui_profile_condition_modal(
            ctx,
            ui,
            &mut args.gui_local.profile_condition_modal,
            &args.config.profiles[profile_idx].name.clone(),
            &mut args.config.profiles[profile_idx].condition,
        );
    }
    if args.gui_local.new_layer_modal_open {
        new_layer_modal(ctx, ui, args, profile_idx);
    }
}

fn layers_table_ui(ui: &mut egui::Ui, args: &mut ReemApp, profile_idx: usize) {
    enum LayerSelect {
        None,
        Base,
        Other(usize),
    }
    use egui_extras::{Column, TableBuilder};
    let header_height = 12.0;
    let row_height = 20.0;
    let btn_size = [20.0, 20.0];
    let mut pointing_hand = false;
    let mut to_delete = None;
    let mut layer_select = LayerSelect::None;
    TableBuilder::new(ui)
        .id_salt("Layers Table")
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
                    ui.label("Base Layer");
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
                    layer_select = LayerSelect::Base;
                }
            });
            // let profiles_len = args.config.profiles.len();
            // let mut to_swap: Option<(usize, usize)> = None;
            for (i, layer) in args.config.profiles[profile_idx]
                .layers
                .iter_mut()
                .enumerate()
            {
                // let first = i == 0;
                // let last = i == profiles_len - 1;
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
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            if ui.add_sized(btn_size, egui::Button::new("✖")).clicked() {
                                to_delete = Some(i);
                            };
                        });
                    });
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.label(&layer.name);
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
                        layer_select = LayerSelect::Other(i);
                    }
                });
            }
            // if let Some((a, b)) = to_swap {
            //     args.config.profiles.swap(a, b);
            // }
        });
    if let Some(to_delete) = to_delete {
        args.config.profiles[profile_idx].layers.remove(to_delete);
    }
    match layer_select {
        LayerSelect::None => (),
        LayerSelect::Base => {
            args.gui_local.menu = GuiMenu::ProfileBaseLayer { profile_idx };
        }
        LayerSelect::Other(i) => {
            args.gui_local.menu = GuiMenu::ProfileLayer {
                profile_idx,
                layer_idx: i,
            }
        }
    }
    if pointing_hand {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
}

fn get_profile_condition_text(condition: &config::ProfileCondition) -> String {
    use config::ProfileCondition as PC;
    match condition {
        PC::TitleAndProcess { title, process } => {
            format!("Active when {title} ({process}) is in focus")
        }
        PC::Title { title } => {
            format!("Active when {title} is in focus")
        }
        PC::Process { process } => {
            format!("Active when the process {process} is in focus")
        }
        PC::OriBF => "Active when Ori and the Blind Forest is in focus".to_string(),
        PC::OriBFDE => {
            "Active when Ori and the Blind Forest: Definitive Edition is in focus".to_string()
        }
        PC::OriWotW => "Active when Ori and the Will of the Wisps is in focus".to_string(),
    }
}

fn new_layer_modal(
    ctx: &egui::Context,
    _ui: &mut egui::Ui,
    args: &mut ReemApp,
    profile_idx: usize,
) {
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
        args.config.profiles[profile_idx].layers.push(new_layer);
        args.gui_local.new_layer_modal_open = false;
    } else if cancel {
        args.gui_local.new_layer_modal_open = false;
    }
}

fn ui_profile_condition_modal(
    ctx: &egui::Context,
    _ui: &mut egui::Ui,
    modal_opts: &mut ProfileConditionModalOpts,
    profile_name: &str,
    condition: &mut config::ProfileCondition,
) {
    let mut ok = false;
    let mut cancel = false;
    let enable_title = match modal_opts.condition {
        ProfileConditionUI::TitleAndProcess | ProfileConditionUI::Title => true,
        _ => false,
    };
    let enable_process = match modal_opts.condition {
        ProfileConditionUI::TitleAndProcess | ProfileConditionUI::Process => true,
        _ => false,
    };
    let enable_table = match modal_opts.condition {
        ProfileConditionUI::TitleAndProcess
        | ProfileConditionUI::Title
        | ProfileConditionUI::Process => true,
        ProfileConditionUI::OriBF | ProfileConditionUI::OriBFDE | ProfileConditionUI::OriWotW => {
            false
        }
    };
    let valid = |modal_opts: &mut ProfileConditionModalOpts| match modal_opts.condition {
        ProfileConditionUI::TitleAndProcess => {
            !modal_opts.title.is_empty() && !modal_opts.process.is_empty()
        }
        ProfileConditionUI::Title => !modal_opts.title.is_empty(),
        ProfileConditionUI::Process => !modal_opts.process.is_empty(),
        ProfileConditionUI::OriBF | ProfileConditionUI::OriBFDE | ProfileConditionUI::OriWotW => {
            true
        }
    };
    let modal = egui::Modal::new(egui::Id::new("Profile Condition Modal")).show(ctx, |ui| {
        ui.heading(format!("Condition for {profile_name}"));
        ui.separator();
        ui.add_space(super::SPACING);

        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            egui::ComboBox::from_label("Condition")
                .selected_text(&modal_opts.condition.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut modal_opts.condition,
                        ProfileConditionUI::TitleAndProcess,
                        "Window Title and Process",
                    );
                    ui.selectable_value(
                        &mut modal_opts.condition,
                        ProfileConditionUI::Title,
                        "Window Title",
                    );
                    ui.selectable_value(
                        &mut modal_opts.condition,
                        ProfileConditionUI::Process,
                        "Process",
                    );
                    ui.separator();
                    ui.selectable_value(
                        &mut modal_opts.condition,
                        ProfileConditionUI::OriBF,
                        "Ori and the Blind Forest",
                    );
                    ui.selectable_value(
                        &mut modal_opts.condition,
                        ProfileConditionUI::OriBFDE,
                        "Ori and the Blind Forest: Definitive Edition",
                    );
                    ui.selectable_value(
                        &mut modal_opts.condition,
                        ProfileConditionUI::OriWotW,
                        "Ori and the Will of the Wisps",
                    );
                });

            ui.add_space(super::SPACING);

            StripBuilder::new(ui)
                .size(Size::exact(300.0))
                .size(Size::exact(20.0))
                .size(Size::exact(27.0)) // OK, Cancel
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        ui.columns_const(|[col_1, col_2]| {
                            col_1.add_enabled_ui(enable_title, |ui| {
                                ui.label("Window Title");
                                ui.text_edit_singleline(&mut modal_opts.title);
                            });

                            col_2.add_enabled_ui(enable_process, |ui| {
                                ui.label("Process");
                                ui.text_edit_singleline(&mut modal_opts.process);
                            });
                        });

                        ui.add_space(super::SPACING);

                        ui.add_enabled_ui(enable_table, |ui| {
                            egui::Frame::new()
                                .stroke(egui::Stroke {
                                    width: 1.0,
                                    color: egui::Color32::DARK_GRAY,
                                })
                                .inner_margin(4.0)
                                .corner_radius(4.0)
                                .show(ui, |ui| {
                                    if let Some(query_windows::WindowInfo { title, process }) =
                                        ui_open_windows_table(ui, &modal_opts.open_windows)
                                    {
                                        modal_opts.title = title;
                                        modal_opts.process = process;
                                    }
                                });
                        });
                    });
                    strip.cell(|ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.add_enabled_ui(enable_table, |ui| {
                                if ui.button("Refresh").clicked() {
                                    modal_opts.open_windows =
                                        query_windows::enumerate_open_windows();
                                }
                            });
                        });
                    });
                    strip.cell(|ui| {
                        ui.separator();
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Cancel").clicked() {
                                cancel = true;
                            }
                            ui.add_enabled_ui(valid(modal_opts), |ui| {
                                if ui.button("OK").clicked() {
                                    ok = true;
                                }
                            });
                        });
                    });
                });
        });
    });
    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
        ok = true;
    }
    if modal.should_close() {
        cancel = true;
    }
    if ok && valid(modal_opts) {
        *condition = match modal_opts.condition {
            ProfileConditionUI::TitleAndProcess => config::ProfileCondition::TitleAndProcess {
                title: modal_opts.title.clone(),
                process: modal_opts.process.clone(),
            },
            ProfileConditionUI::Title => config::ProfileCondition::Title {
                title: modal_opts.title.clone(),
            },
            ProfileConditionUI::Process => config::ProfileCondition::Process {
                process: modal_opts.process.clone(),
            },
            ProfileConditionUI::OriBF => config::ProfileCondition::OriBF,
            ProfileConditionUI::OriBFDE => config::ProfileCondition::OriBFDE,
            ProfileConditionUI::OriWotW => config::ProfileCondition::OriWotW,
        };
        modal_opts.modal_open = false;
    } else if cancel {
        modal_opts.modal_open = false;
    }
}

fn ui_open_windows_table(
    ui: &mut egui::Ui,
    windows: &[query_windows::WindowInfo],
) -> Option<query_windows::WindowInfo> {
    use egui_extras::{Column, TableBuilder};
    let header_height = 12.0;
    let row_height = 20.0;
    let mut pointing_hand = false;
    let mut window_select = None;
    TableBuilder::new(ui)
        .id_salt("Open Windows Table")
        .striped(true)
        .auto_shrink(false)
        .sense(egui::Sense::click_and_drag())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(200.0)) // Process
        .column(Column::remainder()) // Window Title
        .header(header_height, |mut header| {
            header.col(|ui| {
                ui.strong("Process");
            });
            header.col(|ui| {
                ui.strong("Window Title");
            });
        })
        .body(|mut body| {
            for window in windows {
                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.label(&window.process);
                    });
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.label(&window.title);
                    });
                    if row.response().hovered() {
                        pointing_hand = true;
                    }
                    if row.response().clicked() {
                        window_select = Some(window.clone());
                    }
                });
            }
        });
    if pointing_hand {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
    window_select
}
