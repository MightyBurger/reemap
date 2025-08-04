use crate::gui::reemapp::EditProfileModalOpts;
use crate::gui::reemapp::style;
use crate::gui::reemapp::ui_copy_modal::ui_copy_modal;
use crate::gui::reemapp::ui_edit_profile_modal::ui_edit_profile_modal;
use crate::gui::reemapp::ui_ok_cancel_modal::ui_ok_cancel_modal;
use crate::gui::reemapp::ui_tables::ui_enable_clickable_table;
use crate::gui::reemapp::ui_tables::ui_rearrange_table;
use crate::query_windows;

use super::GuiMenu;
use super::ReemApp;

pub fn ui_main(ui: &mut egui::Ui, args: &mut ReemApp) {
    use crate::gui::reemapp::style::REEMAP_SHADOW;
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
                    .size(Size::initial(style::BUTTON_HEIGHT))
                    .size(Size::remainder())
                    .vertical(|mut strip| {
                        strip.empty();
                        strip.cell(|ui| {
                            egui::Frame::new().shadow(REEMAP_SHADOW).show(ui, |ui| {
                                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                    ui.add_space(style::SPACING);
                                    style::UI_FRAME.show(ui, |ui| {
                                        if args.config.profiles.is_empty() {
                                            ui.centered_and_justified(|ui| {
                                                ui.style_mut().interaction.selectable_labels =
                                                    false;
                                                ui.label("Add a profile to get started.");
                                            });
                                        } else {
                                            let profile_select = ui_enable_clickable_table(
                                                ui,
                                                &mut args.config.profiles,
                                                "Profile",
                                            );
                                            if let Some(profile_idx) = profile_select {
                                                args.gui_local.menu =
                                                    GuiMenu::Profile { profile_idx };
                                            }
                                        }
                                    });
                                });
                            });
                        });
                        strip.strip(|builder| {
                            builder
                                .size(Size::remainder())
                                .sizes(Size::initial(style::BUTTON_WIDTH), 3) // 3 buttons
                                .size(Size::remainder())
                                .horizontal(|mut strip| {
                                    strip.empty();
                                    strip.cell(|ui| {
                                        if ui
                                            .add_sized(
                                                style::BUTTON_SIZE,
                                                egui::Button::new("Add Profile"),
                                            )
                                            .clicked()
                                        {
                                            args.gui_local.new_profile_modal =
                                                EditProfileModalOpts {
                                                    modal_open: true,
                                                    open_windows:
                                                        query_windows::enumerate_open_windows(),
                                                    ..Default::default()
                                                };
                                        }
                                    });
                                    strip.cell(|ui| {
                                        ui.add_enabled_ui(!args.config.profiles.is_empty(), |ui| {
                                            if ui
                                                .add_sized(
                                                    style::BUTTON_SIZE,
                                                    egui::Button::new("Copy Profile"),
                                                )
                                                .clicked()
                                            {
                                                args.gui_local.copy_profile_modal = true;
                                            }
                                        });
                                    });
                                    strip.cell(|ui| {
                                        ui.add_enabled_ui(!args.config.profiles.is_empty(), |ui| {
                                            if ui
                                                .add_sized(
                                                    style::BUTTON_SIZE,
                                                    egui::Button::new("Rearrange"),
                                                )
                                                .clicked()
                                            {
                                                args.gui_local.rearrange_profiles_modal.new_order =
                                                    args.config.profiles.clone();
                                                args.gui_local
                                                    .rearrange_profiles_modal
                                                    .modal_open = true;
                                            }
                                        });
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

    // ----- Copy profile modal -----

    if args.gui_local.copy_profile_modal {
        ui_copy_modal(
            ui,
            &mut args.gui_local.copy_profile_modal,
            &mut args.config.profiles,
            "Profile",
        );
    }

    // ----- Rearrange profiles modal -----

    if args.gui_local.rearrange_profiles_modal.modal_open {
        let modal_opts = &mut args.gui_local.rearrange_profiles_modal;
        let profiles = &mut args.config.profiles;
        let ok_cancel = ui_ok_cancel_modal(ui, "", true, |ui| {
            ui.heading("Rearrange and Delete Profiles");
            ui.separator();
            ui.add_space(style::SPACING);

            style::UI_FRAME.show(ui, |ui| {
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
