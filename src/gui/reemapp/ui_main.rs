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
    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        if ui.button("Add Profile").clicked() {
            args.gui_local.new_profile_modal = EditProfileModalOpts {
                modal_open: true,
                name: String::from("New Profile"),
                open_windows: query_windows::enumerate_open_windows(),
                ..Default::default()
            };
        }
        if ui.button("Rearrange").clicked() {
            args.gui_local.rearrange_profiles_modal.new_order = args.config.profiles.clone();
            args.gui_local.rearrange_profiles_modal.modal_open = true;
        }
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.add_space(super::SPACING);
            egui::Frame::new()
                .stroke(egui::Stroke {
                    width: 1.0,
                    color: egui::Color32::DARK_GRAY,
                })
                .inner_margin(4.0)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    if args.config.profiles.len() > 0 {
                        let profile_select =
                            ui_enable_clickable_table(ui, &mut args.config.profiles, "Profiles");
                        if let Some(profile_idx) = profile_select {
                            args.gui_local.menu = GuiMenu::Profile { profile_idx };
                        }
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("Add a profile to get started.");
                        });
                    }
                });
        });
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
