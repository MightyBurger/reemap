use crate::gui::reemapp::style;
use crate::gui::reemapp::ui_ok_cancel_modal::ui_ok_cancel_modal;
use crate::gui::reemapp::{EditProfileModalOpts, ProfileConditionUI};
use crate::query_windows;

pub fn ui_edit_profile_modal(
    ui: &mut egui::Ui,
    modal_opts: &mut EditProfileModalOpts,
    heading: &str,
) -> Option<bool> {
    use egui_extras::{Size, StripBuilder};

    let valid = match modal_opts.condition {
        ProfileConditionUI::TitleAndProcess => {
            !modal_opts.title.is_empty() && !modal_opts.process.is_empty()
        }
        ProfileConditionUI::Title => !modal_opts.title.is_empty(),
        ProfileConditionUI::Process => !modal_opts.process.is_empty(),
        ProfileConditionUI::Always
        | ProfileConditionUI::OriBF
        | ProfileConditionUI::OriBFDE
        | ProfileConditionUI::OriWotW => true,
    };
    let helper_text = if valid {
        modal_opts.clone().extract_condition().helper_text()
    } else {
        String::from("Choose a window title or process")
    };
    ui_ok_cancel_modal(ui, &helper_text, valid, |ui| {
        let enable_title = matches!(
            modal_opts.condition,
            ProfileConditionUI::TitleAndProcess | ProfileConditionUI::Title
        );
        let enable_process = matches!(
            modal_opts.condition,
            ProfileConditionUI::TitleAndProcess | ProfileConditionUI::Process
        );
        let enable_table = match modal_opts.condition {
            ProfileConditionUI::TitleAndProcess
            | ProfileConditionUI::Title
            | ProfileConditionUI::Process => true,
            ProfileConditionUI::Always
            | ProfileConditionUI::OriBF
            | ProfileConditionUI::OriBFDE
            | ProfileConditionUI::OriWotW => false,
        };
        ui.heading(heading);
        ui.separator();

        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            ui.label("Profile name");
            ui.text_edit_singleline(&mut modal_opts.name);
            ui.add_space(style::SPACING);
            ui.label(
                "Reemap decides which profile to use based off what window is in focus. \
                Only one profile is active at a time. \
                Choose a window below.",
            );
            ui.add_space(style::SPACING);

            ui.label("Condition");
            egui::ComboBox::from_id_salt("condition")
                .selected_text(modal_opts.condition.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut modal_opts.condition,
                        ProfileConditionUI::Always,
                        "Always active",
                    );
                    ui.selectable_value(
                        &mut modal_opts.condition,
                        ProfileConditionUI::TitleAndProcess,
                        "Window title and process",
                    );
                    ui.selectable_value(
                        &mut modal_opts.condition,
                        ProfileConditionUI::Title,
                        "Window title",
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

            ui.add_space(style::SPACING);

            StripBuilder::new(ui)
                .size(Size::exact(300.0))
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

                        ui.add_space(style::SPACING);

                        ui.add_enabled_ui(enable_table, |ui| {
                            style::UI_FRAME.show(ui, |ui| {
                                if let Some(query_windows::WindowInfo {
                                    title,
                                    process,
                                    rect: _,
                                }) = ui_open_windows_table(ui, &modal_opts.open_windows)
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
                                if ui
                                    .add_sized(style::BUTTON_SIZE, egui::Button::new("Refresh"))
                                    .clicked()
                                {
                                    modal_opts.open_windows =
                                        query_windows::enumerate_open_windows();
                                }
                            });
                        });
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

    let mut pointing_hand = false;
    let mut window_select = None;
    TableBuilder::new(ui)
        .id_salt("Open Windows Table")
        .striped(true)
        .auto_shrink(false)
        .sense(egui::Sense::click_and_drag())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(200.0)) // Process
        .column(Column::remainder().at_most(420.0)) // Window Title
        .header(style::HEADER_HEIGHT, |mut header| {
            header.col(|ui| {
                ui.strong("Process");
            });
            header.col(|ui| {
                ui.strong("Window Title");
            });
        })
        .body(|mut body| {
            for window in windows {
                body.row(style::ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.add(egui::Label::new(&window.process).truncate());
                    });
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.add(egui::Label::new(&window.title).truncate());
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
