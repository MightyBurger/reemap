// Returns:
//  None if no action taken
//  Some(false) if cancelled
//  Some(true) if accepted
pub fn ui_ok_cancel_modal(
    //
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui),
) -> Option<bool> {
    use egui_extras::{Size, StripBuilder};

    let ok_cancel_width = 60.0;
    let mut ok = false;
    let mut cancel = false;

    let modal = egui::Modal::new(egui::Id::new("rearrange profiles modal")).show(ui.ctx(), |ui| {
        StripBuilder::new(ui)
            .size(Size::exact(400.0))
            .size(Size::exact(20.0))
            .vertical(|mut strip| {
                strip.cell(add_contents);
                strip.cell(|ui| {
                    ui.separator();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_sized(
                                [ok_cancel_width, ui.available_height()],
                                egui::Button::new("Cancel"),
                            )
                            .clicked()
                        {
                            cancel = true;
                        }
                        if ui
                            .add_sized(
                                [ok_cancel_width, ui.available_height()],
                                egui::Button::new("OK"),
                            )
                            .clicked()
                        {
                            ok = true;
                        }
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
    if ok {
        Some(true)
    } else if cancel {
        Some(false)
    } else {
        None
    }
}
