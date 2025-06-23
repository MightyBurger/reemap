use super::GuiMenu;
use super::ReemApp;
use crate::buttons;
use strum::IntoEnumIterator;

pub fn ui_profile_layer(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    args: &mut ReemApp,
    profile_idx: usize,
    layer_idx: usize,
) {
    enum BreadcrumbClick {
        ClickedProfiles,
        ClickedOpenProfile,
    }
    let mut clicked = None;
    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                let profiles_breadcrumb_response = ui.add(
                    egui::Label::new(egui::RichText::new("Profiles").heading())
                        .sense(egui::Sense::click()),
                );
                if profiles_breadcrumb_response.hovered() {
                    ui.ctx()
                        .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }
                if profiles_breadcrumb_response.clicked() {
                    clicked = Some(BreadcrumbClick::ClickedProfiles);
                }

                ui.heading(" > ");

                let openprofile_breadcrumb_response = ui.add(
                    egui::Label::new(
                        egui::RichText::new(format!("{}", &args.config.profiles[profile_idx].name))
                            .heading(),
                    )
                    .sense(egui::Sense::click()),
                );
                if openprofile_breadcrumb_response.hovered() {
                    ui.ctx()
                        .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }
                if openprofile_breadcrumb_response.clicked() {
                    clicked = Some(BreadcrumbClick::ClickedOpenProfile);
                }

                ui.heading(" > ");
                ui.heading(format!(
                    "{}",
                    &args.config.profiles[profile_idx].layers[layer_idx].name
                ));
            });
            ui.separator();
            ui.add_space(super::SPACING);

            // ui.label("Profile Name");
            // ui.text_edit_singleline(&mut args.get_open_profile_ui_mut().unwrap().name);
            // ui.add_space(super::SPACING);

            egui::Frame::new()
                .stroke(egui::Stroke {
                    width: 1.0,
                    color: egui::Color32::DARK_GRAY,
                })
                .inner_margin(4.0)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    remaps_table(ui, args, profile_idx, layer_idx);
                });
        });
    });
    match clicked {
        None => (),
        Some(BreadcrumbClick::ClickedProfiles) => {
            args.gui_local.menu = GuiMenu::MainMenu;
        }
        Some(BreadcrumbClick::ClickedOpenProfile) => {
            args.gui_local.menu = GuiMenu::ProfileMenu {
                profile_idx: profile_idx,
            }
        }
    }
}
fn remaps_table(ui: &mut egui::Ui, args: &mut ReemApp, profile_idx: usize, layer_idx: usize) {
    use egui_extras::{Column, TableBuilder};
    let header_height = 12.0;
    let row_height = 20.0;
    let btn_size = [20.0, 20.0];
    let mut pointing_hand = false;
    let mut button_select = None;
    TableBuilder::new(ui)
        .id_salt("Layers Table")
        .striped(true)
        .auto_shrink(false)
        .sense(egui::Sense::click_and_drag())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(120.0)) // Enabled
        .column(Column::remainder()) // Profile Name
        .header(header_height, |mut header| {
            header.col(|ui| {
                ui.strong("Input");
            });
            header.col(|ui| {
                ui.strong("Output");
            });
        })
        .body(|mut body| {
            let key_iter = buttons::key::KeyButton::iter().map(|key| buttons::Button::from(key));
            let mouse_iter =
                buttons::mouse::MouseButton::iter().map(|mouse| buttons::Button::from(mouse));
            let wheel_iter =
                buttons::wheel::MouseWheelButton::iter().map(|wheel| buttons::Button::from(wheel));

            for button in key_iter.chain(mouse_iter).chain(wheel_iter) {
                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.label(format!("{button}"));
                    });
                    row.col(|ui| {
                        let policy = args.config.profiles[profile_idx].layers[layer_idx].policy
                            [button]
                            .clone();
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.label(format!("{policy}"));
                    });
                    if row.response().hovered() {
                        pointing_hand = true;
                    }
                    if row.response().clicked() {
                        button_select = Some(button);
                    }
                });
            }
        });
    // todo: handle button_select
    if pointing_hand {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
}
