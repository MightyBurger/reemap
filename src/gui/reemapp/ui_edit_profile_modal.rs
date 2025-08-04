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

    let valid = modal_opts.valid();
    let helper_text = if valid {
        modal_opts.clone().extract_condition().helper_text()
    } else if modal_opts.name.is_empty() {
        String::from("Choose a profile name")
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

        let min_width = 80.0;

        ui.add_space(style::SPACING);
        egui::Grid::new("edit_profile1")
            .min_col_width(min_width)
            .num_columns(2)
            .spacing([style::SPACING, style::SPACING])
            .show(ui, |ui| {
                ui.label("Profile name");
                ui.add(
                    egui::TextEdit::singleline(&mut modal_opts.name)
                        .hint_text("Insert profile name"),
                );
                ui.end_row();

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

                        ui.scope(|ui| {
                            ui.style_mut().interaction.selectable_labels = false;
                            ui.add(egui::Label::new(egui::RichText::new("Presets").italics()));
                        });

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
                ui.end_row();
            });
        ui.add_space(style::SPACING * 2.0);

        ui.add_enabled(
            enable_table,
            egui::Label::new("Select from a list of running applications:"),
        );

        StripBuilder::new(ui)
            .size(Size::exact(200.0))
            .size(Size::initial(style::BUTTON_HEIGHT))
            .size(Size::exact(style::SPACING * 2.0))
            .size(Size::initial(style::BUTTON_HEIGHT * 2.0))
            .size(Size::exact(style::SPACING))
            .size(Size::initial(1.0))
            .vertical(|mut strip| {
                strip.cell(|ui| {
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
                    ui.add_enabled_ui(enable_table, |ui| {
                        if ui
                            .add_sized(style::BUTTON_SIZE, egui::Button::new("Refresh"))
                            .clicked()
                        {
                            modal_opts.open_windows = query_windows::enumerate_open_windows();
                        }
                    });
                });
                strip.empty();
                strip.cell(|ui| {
                    egui::Grid::new("edit_profile2")
                        .min_col_width(min_width)
                        .num_columns(2)
                        .spacing([style::SPACING, style::SPACING])
                        .show(ui, |ui| {
                            ui.add_enabled(enable_title, egui::Label::new("Window Title"));
                            ui.add_enabled(
                                enable_title,
                                egui::TextEdit::singleline(&mut modal_opts.title)
                                    .hint_text("Insert title"),
                            );
                            ui.end_row();
                            ui.add_enabled(enable_process, egui::Label::new("Process"));
                            ui.add_enabled(
                                enable_process,
                                egui::TextEdit::singleline(&mut modal_opts.process)
                                    .hint_text("Insert process"),
                            );
                            ui.end_row();
                        });
                });
                strip.empty();
                strip.cell(|ui| {
                    ui.separator();
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
