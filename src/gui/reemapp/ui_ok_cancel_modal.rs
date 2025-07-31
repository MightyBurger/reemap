use crate::gui::reemapp::style;

// Returns:
//  None if no action taken
//  Some(false) if cancelled
//  Some(true) if accepted
pub fn ui_ok_cancel_modal(
    ui: &mut egui::Ui,
    helper_text: &str,
    enable_ok: bool,
    add_contents: impl FnOnce(&mut egui::Ui),
) -> Option<bool> {
    use egui_extras::{Size, StripBuilder};

    let mut ok = false;
    let mut cancel = false;

    let modal = egui::Modal::new(egui::Id::new("ok-cancel modal"))
        .backdrop_color(style::MODAL_BACKDROP_COLOR)
        .frame(style::MODAL_FRAME)
        .show(ui.ctx(), |ui| {
            style::set_reemap_style(ui);
            // Max width and height are arbitrary, but some limit is required. Change when needed.
            ui.set_max_width(650.0);
            ui.set_max_height(550.0);
            StripBuilder::new(ui)
                .size(Size::exact(400.0))
                .size(Size::exact(style::SPACING))
                .size(Size::initial(style::BUTTON_HEIGHT).at_most(style::BUTTON_HEIGHT + 5.0))
                .vertical(|mut strip| {
                    strip.cell(add_contents);
                    strip.empty();
                    strip.strip(|builder| {
                        builder
                            .size(Size::remainder())
                            .size(Size::initial(style::BUTTON_WIDTH * 2.0)) // 2 for two buttons
                            .horizontal(|mut strip| {
                                strip.cell(|ui| {
                                    ui.with_layout(
                                        egui::Layout::left_to_right(egui::Align::BOTTOM),
                                        |ui| {
                                            ui.add(egui::Label::new(helper_text).truncate());
                                        },
                                    );
                                });
                                strip.cell(|ui| {
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui
                                                .add_sized(
                                                    style::BUTTON_SIZE,
                                                    egui::Button::new("Cancel"),
                                                )
                                                .clicked()
                                            {
                                                cancel = true;
                                            }
                                            ui.add_enabled_ui(enable_ok, |ui| {
                                                if ui
                                                    .add_sized(
                                                        style::BUTTON_SIZE,
                                                        egui::Button::new("OK"),
                                                    )
                                                    .clicked()
                                                {
                                                    ok = true;
                                                }
                                            });
                                        },
                                    );
                                });
                            });
                    });
                });
        });

    if ui.ctx().input(|i| i.key_pressed(egui::Key::Enter)) {
        ok = true;
    }
    if modal.should_close() {
        cancel = true;
    }
    if ok && enable_ok {
        Some(true)
    } else if cancel {
        Some(false)
    } else {
        None
    }
}
