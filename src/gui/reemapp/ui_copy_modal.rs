use crate::config;
use crate::gui::reemapp::style;
use crate::gui::reemapp::ui_tables::{ClickListItem, ui_clickable_table};

pub trait CopyListItem: ClickListItem + Clone {
    fn clone_new_name(&self) -> Self;
}

impl CopyListItem for config::Profile {
    fn clone_new_name(&self) -> Self {
        let mut new = self.clone();
        new.name.push_str(" (copy)");
        new
    }
}

impl CopyListItem for config::Layer {
    fn clone_new_name(&self) -> Self {
        let mut new = self.clone();
        new.name.push_str(" (copy)");
        new
    }
}

pub fn ui_copy_modal<T>(
    ui: &mut egui::Ui,
    modal_opts: &mut bool,
    list: &mut Vec<T>,
    list_name: &str,
) where
    T: CopyListItem,
{
    let modal_response = ui_cancel_modal(ui, |ui| {
        ui.heading(format!("Copy {list_name}"));
        ui.separator();
        ui.add_space(super::SPACING);
        style::UI_FRAME.show(ui, |ui| {
            let item_select = ui_clickable_table(ui, list, list_name);
            if let Some(item_select) = item_select {
                let new_item = list[item_select].clone_new_name();
                list.insert(item_select + 1, new_item); // place new item right after the old one
                *modal_opts = false;
            }
        });
    });
    if modal_response {
        *modal_opts = false;
    }
}

fn ui_cancel_modal(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) -> bool {
    use super::BUTTON_HEIGHT;
    use super::BUTTON_SIZE;
    use super::SPACING;
    use crate::gui::reemapp::style;
    use egui_extras::{Size, StripBuilder};

    let mut close = false;

    let modal = egui::Modal::new(egui::Id::new("cancel modal"))
        .backdrop_color(style::MODAL_BACKDROP_COLOR)
        .frame(style::MODAL_FRAME)
        .show(ui.ctx(), |ui| {
            style::set_reemap_style(ui);
            // Max width and height are arbitrary, but some limit is required. Change when needed.
            ui.set_max_width(650.0);
            ui.set_max_height(550.0);
            StripBuilder::new(ui)
                .size(Size::exact(400.0))
                .size(Size::exact(SPACING))
                .size(Size::initial(BUTTON_HEIGHT).at_most(BUTTON_HEIGHT + 5.0))
                .vertical(|mut strip| {
                    strip.cell(add_contents);
                    strip.empty();
                    strip.cell(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add_sized(BUTTON_SIZE, egui::Button::new("Cancel"))
                                .clicked()
                            {
                                close = true;
                            }
                        });
                    });
                });
        });

    if modal.should_close() {
        close = true;
    }
    close
}
