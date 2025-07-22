use crate::gui::reemapp::EditProfileModalOpts;
use crate::gui::reemapp::SPACING;
use crate::gui::reemapp::ui_edit_profile_modal::ui_edit_profile_modal;
use crate::gui::reemapp::ui_ok_cancel_modal::ui_ok_cancel_modal;
use crate::gui::reemapp::ui_tables::ui_enable_clickable_table;
use crate::gui::reemapp::ui_tables::ui_rearrange_table;
use crate::query_windows;

use super::GuiMenu;
use super::ReemApp;

pub fn ui_main(ui: &mut egui::Ui, args: &mut ReemApp) {
    use super::BUTTON_HEIGHT;
    use super::BUTTON_SIZE;
    use super::BUTTON_WIDTH;
    use egui_extras::{Size, StripBuilder};

    StripBuilder::new(ui)
        .size(Size::remainder())
        .size(Size::relative(0.6)) // fraction of width the profile table takes up
        .size(Size::remainder())
        .horizontal(|mut strip| {
            strip.empty();
            strip.cell(|ui| {
                StripBuilder::new(ui)
                    .size(Size::remainder())
                    .size(Size::relative(0.8))
                    .size(Size::initial(BUTTON_HEIGHT))
                    .size(Size::remainder())
                    .vertical(|mut strip| {
                        strip.empty();
                        strip.cell(|ui| {
                            egui::Frame::new()
                                .shadow(egui::Shadow {
                                    offset: [0, 0],
                                    blur: 16,
                                    spread: 8,
                                    color: egui::Color32::from_black_alpha(128),
                                })
                                .show(ui, |ui| {
                                    ui.with_layout(
                                        egui::Layout::top_down(egui::Align::Center),
                                        |ui| {
                                            ui.add_space(super::SPACING);
                                            egui::Frame::new()
                                                .stroke(egui::Stroke {
                                                    width: 1.0,
                                                    color: egui::Color32::DARK_GRAY,
                                                })
                                                .inner_margin(4.0)
                                                .corner_radius(4.0)
                                                .show(ui, |ui| {
                                                    if args.config.profiles.is_empty() {
                                                        ui.centered_and_justified(|ui| {
                                                            ui.style_mut()
                                                                .interaction
                                                                .selectable_labels = false;
                                                            ui.label(
                                                                "Add a profile to get started.",
                                                            );
                                                        });
                                                    } else {
                                                        let profile_select =
                                                            ui_enable_clickable_table(
                                                                ui,
                                                                &mut args.config.profiles,
                                                                "Profiles",
                                                            );
                                                        if let Some(profile_idx) = profile_select {
                                                            args.gui_local.menu =
                                                                GuiMenu::Profile { profile_idx };
                                                        }
                                                    }
                                                });
                                        },
                                    );
                                });
                        });
                        strip.strip(|builder| {
                            builder
                                .size(Size::remainder())
                                .sizes(Size::initial(BUTTON_WIDTH), 2) // 2 buttons
                                .size(Size::remainder())
                                .horizontal(|mut strip| {
                                    strip.empty();
                                    strip.cell(|ui| {
                                        if ui
                                            .add_sized(
                                                BUTTON_SIZE,
                                                egui::Button::new("Add Profile"),
                                            )
                                            .clicked()
                                        {
                                            args.gui_local.new_profile_modal =
                                                EditProfileModalOpts {
                                                    modal_open: true,
                                                    name: String::from("New Profile"),
                                                    open_windows:
                                                        query_windows::enumerate_open_windows(),
                                                    ..Default::default()
                                                };
                                        }
                                    });
                                    strip.cell(|ui| {
                                        if ui
                                            .add_sized(BUTTON_SIZE, egui::Button::new("Rearrange"))
                                            .clicked()
                                        {
                                            args.gui_local.rearrange_profiles_modal.new_order =
                                                args.config.profiles.clone();
                                            args.gui_local.rearrange_profiles_modal.modal_open =
                                                true;
                                        }
                                    });
                                    strip.empty();
                                });
                        });
                        strip.empty();
                    });
            });
            strip.empty();
        });

    // ----- New profile modal -----

    if args.gui_local.new_profile_modal.modal_open {
        let ok_cancel = ui_edit_profile_modal(
            ui,
            &mut args.gui_local.new_profile_modal,
            "Create new profile",
        );
        match ok_cancel {
            Some(true) => {
                args.config
                    .profiles
                    .push(args.gui_local.new_profile_modal.clone().into());
                args.gui_local.new_profile_modal.modal_open = false;
            }
            Some(false) => {
                args.gui_local.new_profile_modal.modal_open = false;
            }
            None => (),
        }
    }

    // ----- Rearrange profiles modal -----

    if args.gui_local.rearrange_profiles_modal.modal_open {
        let modal_opts = &mut args.gui_local.rearrange_profiles_modal;
        let profiles = &mut args.config.profiles;
        let ok_cancel = ui_ok_cancel_modal(ui, |ui| {
            ui.heading("Rearrange and Delete Profiles");
            ui.separator();
            ui.add_space(SPACING);

            egui::Frame::new()
                .stroke(egui::Stroke {
                    width: 1.0,
                    color: egui::Color32::DARK_GRAY,
                })
                .inner_margin(4.0)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui_rearrange_table(ui, &mut modal_opts.new_order, "Profile");
                });
        });
        match ok_cancel {
            Some(true) => {
                *profiles = modal_opts.new_order.clone();
                modal_opts.modal_open = false;
            }
            Some(false) => {
                modal_opts.modal_open = false;
            }
            None => (),
        }
    }
}
