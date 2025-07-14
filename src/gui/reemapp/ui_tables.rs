use crate::buttons;
use crate::config;
use smallvec::SmallVec;
use strum::IntoEnumIterator;

pub trait EnableListItem: std::fmt::Display {
    fn enable_mut(&mut self) -> &mut bool;
}

impl EnableListItem for config::Profile {
    fn enable_mut(&mut self) -> &mut bool {
        &mut self.enabled
    }
}

impl EnableListItem for config::Layer {
    fn enable_mut(&mut self) -> &mut bool {
        &mut self.enabled
    }
}

/// Display a table that allows the user to enable items in the list and to click them.
/// Important: if called multiple times within the same `Ui`, each call must have a different
/// `name`, or runtime errors will occur.
/// Returns the index of the item the user clicked.
pub fn ui_enable_clickable_table<T>(
    ui: &mut egui::Ui,
    list: &mut Vec<T>,
    name: &str,
) -> Option<usize>
where
    T: EnableListItem,
{
    use egui_extras::{Column, TableBuilder};
    let header_height = 12.0;
    let row_height = 20.0;
    let mut pointing_hand = false;
    let mut selected = None;
    TableBuilder::new(ui)
        .id_salt(format!("layers table for {name}"))
        .striped(true)
        .auto_shrink(false)
        .sense(egui::Sense::click_and_drag())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(60.0)) // Enabled
        .column(Column::remainder()) // Profile Name
        .header(header_height, |mut header| {
            header.col(|ui| {
                ui.strong("Enabled");
            });
            header.col(|ui| {
                ui.strong(name);
            });
        })
        .body(|mut body| {
            for (i, item) in list.iter_mut().enumerate() {
                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.with_layout(
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                ui.add(egui::Checkbox::without_text(item.enable_mut()));
                            },
                        );
                    });
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.label(item.to_string());
                    });
                    if row.response().hovered() {
                        pointing_hand = true;
                    }
                    if row.response().clicked() {
                        selected = Some(i);
                    }
                });
            }
        });
    if pointing_hand {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
    selected
}

pub trait RearrangeableListItem: std::fmt::Display {}
impl RearrangeableListItem for config::Profile {}
impl RearrangeableListItem for config::Layer {}
impl RearrangeableListItem for buttons::Button {}
impl RearrangeableListItem for buttons::HoldButton {}

// So I can call ui_rearrangeable_table with either a Vec or a SmallVec.
// TODO: Is there a more idiomatic Rust way to do this?
pub trait RearrangeableList<T> {
    fn as_mut_slice(&mut self) -> &mut [T];
    fn as_slice(&self) -> &[T];
    fn remove(&mut self, index: usize) -> T;
}

impl<T> RearrangeableList<T> for Vec<T> {
    fn as_mut_slice(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
    fn as_slice(&self) -> &[T] {
        self.as_slice()
    }
    fn remove(&mut self, index: usize) -> T {
        self.remove(index)
    }
}

impl<A, T> RearrangeableList<T> for SmallVec<A>
where
    A: smallvec::Array<Item = T>,
{
    fn as_mut_slice(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
    fn as_slice(&self) -> &[T] {
        self.as_slice()
    }
    fn remove(&mut self, index: usize) -> T {
        self.remove(index)
    }
}

/// Display a table that allows the user to re-arrange or delete items in the list.
/// Important: if called multiple times within the same `Ui`, each call must have a different
/// `name`, or runtime errors will occur.
pub fn ui_rearrange_table<T, L>(ui: &mut egui::Ui, list: &mut L, name: &str)
where
    L: RearrangeableList<T>,
    T: RearrangeableListItem,
{
    use egui_extras::{Column, TableBuilder};
    let header_height = 12.0;
    let row_height = 20.0;
    let btn_size = [20.0, 20.0];
    let mut to_delete = None;
    let list_len = list.as_slice().len();
    let mut to_swap: Option<(usize, usize)> = None;
    TableBuilder::new(ui)
        .id_salt(format!("ui_rearrange table for {name}"))
        .striped(true)
        .auto_shrink(false)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder()) // Profile Name
        .column(Column::exact(70.0))
        .header(header_height, |mut header| {
            header.col(|ui| {
                ui.strong(name);
            });
            header.col(|ui| {
                ui.strong("Move");
            });
        })
        .body(|mut body| {
            for (i, item) in list.as_mut_slice().iter_mut().enumerate() {
                let first = i == 0;
                let last = i == list_len - 1;
                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.label(item.to_string());
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
                });
            }
        });
    if let Some((a, b)) = to_swap {
        list.as_mut_slice().swap(a, b);
    }
    if let Some(to_delete) = to_delete {
        list.remove(to_delete);
    }
}

pub fn ui_available_remaps_table(ui: &mut egui::Ui, remaps: &mut config::Output) {
    use egui_extras::{Column, TableBuilder};
    let header_height = 12.0;
    let row_height = 20.0;
    let mut button_select = None;
    let mut pointing_hand = false;
    TableBuilder::new(ui)
        .id_salt("Available Remaps Table")
        .striped(true)
        .auto_shrink(false)
        .sense(egui::Sense::click_and_drag())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder()) // Button Name
        .header(header_height, |mut header| {
            header.col(|ui| {
                ui.strong("Button");
            });
        })
        .body(|mut body| {
            let key_iter = buttons::key::KeyButton::iter().map(buttons::Button::from);
            let mouse_iter = buttons::mouse::MouseButton::iter().map(buttons::Button::from);
            let wheel_iter = buttons::wheel::MouseWheelButton::iter().map(buttons::Button::from);

            for button in key_iter.chain(mouse_iter).chain(wheel_iter) {
                let enabled = !remaps.contains(&button);
                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.add_enabled(enabled, egui::Label::new(format!("{button}")));
                    });
                    if enabled && row.response().hovered() {
                        pointing_hand = true;
                    }
                    if enabled && row.response().clicked() {
                        button_select = Some(button);
                    }
                });
            }
        });
    if pointing_hand {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
    if let Some(button_select) = button_select {
        if !remaps.contains(&button_select) {
            remaps.push(button_select);
        }
    }
}
