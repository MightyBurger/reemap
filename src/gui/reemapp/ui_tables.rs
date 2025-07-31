use crate::buttons;
use crate::config;
use smallvec::SmallVec;
use strum::IntoEnumIterator;

// So I can call table functions with either a Vec or a SmallVec.
pub trait TableList<T> {
    fn as_mut_slice(&mut self) -> &mut [T];
    fn as_slice(&self) -> &[T];
    fn remove(&mut self, index: usize) -> T;
    fn push(&mut self, value: T);
}

impl<T> TableList<T> for Vec<T> {
    fn as_mut_slice(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
    fn as_slice(&self) -> &[T] {
        self.as_slice()
    }
    fn remove(&mut self, index: usize) -> T {
        self.remove(index)
    }
    fn push(&mut self, value: T) {
        self.push(value)
    }
}

impl<A, T> TableList<T> for SmallVec<A>
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
    fn push(&mut self, value: T) {
        self.push(value)
    }
}

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
pub fn ui_enable_clickable_table<T>(ui: &mut egui::Ui, list: &mut [T], name: &str) -> Option<usize>
where
    T: EnableListItem,
{
    use super::HEADER_HEIGHT;
    use super::ROW_HEIGHT;
    use egui_extras::{Column, TableBuilder};

    let mut pointing_hand = false;
    let mut selected = None;
    TableBuilder::new(ui)
        .id_salt(format!("enable-clickable table for {name}"))
        .striped(true)
        .auto_shrink(false)
        .sense(egui::Sense::click_and_drag())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(60.0)) // Enabled
        .column(Column::remainder()) // Profile Name
        .header(HEADER_HEIGHT, |mut header| {
            header.col(|ui| {
                ui.strong("Enabled");
            });
            header.col(|ui| {
                ui.strong(name);
            });
        })
        .body(|mut body| {
            for (i, item) in list.iter_mut().enumerate() {
                body.row(ROW_HEIGHT, |mut row| {
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
                        ui.add(egui::Label::new(item.to_string()).truncate());
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

pub trait ClickListItem: std::fmt::Display {}
impl ClickListItem for config::Profile {}
impl ClickListItem for config::Layer {}

/// Display a table that allows the user to click on an item in the list.
/// Important: if called multiple times within the same `Ui`, each call must have a different
/// `name`, or runtime errors will occur.
/// Returns the index of the item the user clicked.
pub fn ui_clickable_table<T>(ui: &mut egui::Ui, list: &[T], name: &str) -> Option<usize>
where
    T: ClickListItem,
{
    use super::HEADER_HEIGHT;
    use super::ROW_HEIGHT;
    use egui_extras::{Column, TableBuilder};

    let mut pointing_hand = false;
    let mut selected = None;
    TableBuilder::new(ui)
        .id_salt(format!("clickable table for {name}"))
        .striped(true)
        .auto_shrink(false)
        .sense(egui::Sense::click_and_drag())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder()) // Item Name
        .header(HEADER_HEIGHT, |mut header| {
            header.col(|ui| {
                ui.strong(name);
            });
        })
        .body(|mut body| {
            for (i, item) in list.iter().enumerate() {
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.add(egui::Label::new(item.to_string()).truncate());
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

/// Display a table that allows the user to re-arrange or delete items in the list.
/// Important: if called multiple times within the same `Ui`, each call must have a different
/// `name`, or runtime errors will occur.
pub fn ui_rearrange_table<T, L>(ui: &mut egui::Ui, list: &mut L, name: &str)
where
    L: TableList<T>,
    T: RearrangeableListItem,
{
    use super::HEADER_HEIGHT;
    use super::ROW_HEIGHT;
    use egui_extras::{Column, TableBuilder};

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
        .header(HEADER_HEIGHT, |mut header| {
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
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.add(egui::Label::new(item.to_string()).truncate());
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

/// Wrapper around ui_available_inputs_table for buttons.
pub fn ui_available_buttons_table<L>(
    ui: &mut egui::Ui,
    outputs: &mut L,
    search: &str,
    show_rare_keys: bool,
) where
    L: TableList<buttons::Button>,
{
    use buttons::key::KeyType;
    let key_iter = buttons::key::KeyButton::iter()
        .filter(|key| match (show_rare_keys, key.key_type()) {
            (true, KeyType::Common | KeyType::Rare) => true,
            (false, KeyType::Common) => true,
            _ => false,
        })
        .map(buttons::Button::from);
    let mouse_iter = buttons::mouse::MouseButton::iter().map(buttons::Button::from);
    let wheel_iter = buttons::wheel::MouseWheelButton::iter().map(buttons::Button::from);
    let button_iter = mouse_iter
        .chain(wheel_iter)
        .chain(key_iter)
        .filter(|button| {
            let mod_search = search.trim().to_lowercase();
            mod_search.is_empty() || button.to_string().to_lowercase().contains(&mod_search)
        });
    ui_available_inputs_table(ui, button_iter, outputs);
}

/// Wrapper around ui_available_inputs_table for hold buttons.
pub fn ui_available_hold_buttons_table<L>(
    ui: &mut egui::Ui,
    outputs: &mut L,
    search: &str,
    show_rare_keys: bool,
) where
    L: TableList<buttons::HoldButton>,
{
    use buttons::key::KeyType;
    let key_iter = buttons::key::KeyButton::iter()
        .filter(|key| match (show_rare_keys, key.key_type()) {
            (true, KeyType::Common | KeyType::Rare) => true,
            (false, KeyType::Common) => true,
            _ => false,
        })
        .map(buttons::HoldButton::from);
    let mouse_iter = buttons::mouse::MouseButton::iter().map(buttons::HoldButton::from);
    let button_iter = mouse_iter.chain(key_iter).filter(|button| {
        let mod_search = search.trim().to_lowercase();
        mod_search.is_empty() || button.to_string().to_lowercase().contains(&mod_search)
    });
    ui_available_inputs_table(ui, button_iter, outputs);
}

/// A table that lists inputs you can add to a list of outputs.
pub fn ui_available_inputs_table<L, I, T>(ui: &mut egui::Ui, inputs: I, outputs: &mut L)
where
    L: TableList<T>,
    I: Iterator<Item = T>,
    T: buttons::Input,
{
    use super::HEADER_HEIGHT;
    use super::ROW_HEIGHT;
    use egui_extras::{Column, TableBuilder};

    let mut button_select = None;
    let mut pointing_hand = false;
    TableBuilder::new(ui)
        .id_salt("Available Inputs Table")
        .striped(true)
        .auto_shrink(false)
        .sense(egui::Sense::click_and_drag())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(60.0)) // Device
        .column(Column::remainder()) // Button Name
        .header(HEADER_HEIGHT, |mut header| {
            header.col(|ui| {
                ui.strong("Device");
            });
            header.col(|ui| {
                ui.strong("Button");
            });
        })
        .body(|mut body| {
            for input in inputs {
                let enabled = !outputs.as_slice().contains(&input);
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        let device = input.device();
                        ui.add_enabled(enabled, egui::Label::new(device));
                    });
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.add_enabled(enabled, egui::Label::new(input.to_string()));
                    });
                    if enabled && row.response().hovered() {
                        pointing_hand = true;
                    }
                    if enabled && row.response().clicked() {
                        button_select = Some(input);
                    }
                });
            }
        });
    if pointing_hand {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
    if let Some(button_select) = button_select {
        if !outputs.as_slice().contains(&button_select) {
            outputs.push(button_select);
        }
    }
}
