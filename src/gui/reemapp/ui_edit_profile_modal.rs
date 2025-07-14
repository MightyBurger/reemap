use crate::gui::reemapp::ui_ok_cancel_modal::ui_ok_cancel_modal;
use crate::gui::reemapp::{EditProfileModalOpts, ProfileConditionUI, SPACING};
use crate::query_windows;

pub fn ui_edit_profile_modal(
    ui: &mut egui::Ui,
    modal_opts: &mut EditProfileModalOpts,
    heading: &str,
) -> Option<bool> {
    use egui_extras::{Size, StripBuilder};
    ui_ok_cancel_modal(ui, |ui| {
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
            ProfileConditionUI::OriBF
            | ProfileConditionUI::OriBFDE
            | ProfileConditionUI::OriWotW => false,
        };
        // TODO: use again
        let valid = |modal_opts: &mut EditProfileModalOpts| match modal_opts.condition {
            ProfileConditionUI::TitleAndProcess => {
                !modal_opts.title.is_empty() && !modal_opts.process.is_empty()
            }
            ProfileConditionUI::Title => !modal_opts.title.is_empty(),
            ProfileConditionUI::Process => !modal_opts.process.is_empty(),
            ProfileConditionUI::OriBF
            | ProfileConditionUI::OriBFDE
            | ProfileConditionUI::OriWotW => true,
        };
        ui.heading(heading);
        ui.separator();
        // ui.add_space(super::SPACING);

        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            ui.label("Profile name");
            ui.text_edit_singleline(&mut modal_opts.name);
            ui.separator();

            ui.label("Condition");
            egui::ComboBox::from_id_salt("condition")
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
                .size(Size::exact(20.0))
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
                        ui.add_space(SPACING);
                        ui.label(modal_opts.clone().extract_condition().helper_text());
                    });
                });
        });
    })
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
