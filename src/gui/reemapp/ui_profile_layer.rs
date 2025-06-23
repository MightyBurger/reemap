use super::GuiMenu;
use super::ReemApp;

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

            // egui::Frame::new()
            //     .stroke(egui::Stroke {
            //         width: 1.0,
            //         color: egui::Color32::DARK_GRAY,
            //     })
            //     .inner_margin(4.0)
            //     .corner_radius(4.0)
            //     .show(ui, |ui| {
            //         layers_table_ui(ui, args);
            //     });
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
// fn remaps_table(ui: &mut egui::Ui, args: &mut ReemApp) {
//     enum LayerSelect {
//         None,
//         Base,
//         Other(usize),
//     }
//     use egui_extras::{Column, TableBuilder};
//     let header_height = 12.0;
//     let row_height = 20.0;
//     let btn_size = [20.0, 20.0];
//     let mut pointing_hand = false;
//     let mut to_delete = None;
//     let mut layer_select = LayerSelect::None;
//     TableBuilder::new(ui)
//         .id_salt("Layers Table")
//         .striped(true)
//         .auto_shrink(false)
//         .sense(egui::Sense::click_and_drag())
//         .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
//         .column(Column::exact(60.0)) // Enabled
//         .column(Column::exact(60.0)) // Delete
//         .column(Column::remainder()) // Profile Name
//         // .column(Column::exact(60.0))
//         .header(header_height, |mut header| {
//             header.col(|ui| {
//                 ui.strong("Enabled");
//             });
//             header.col(|ui| {
//                 ui.strong("Remove");
//             });
//             header.col(|ui| {
//                 ui.strong("Name");
//             });
//         })
//         .body(|mut body| {
//             body.row(row_height, |mut row| {
//                 let mut dummy = true;
//                 row.col(|ui| {
//                     ui.with_layout(
//                         egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
//                         |ui| {
//                             ui.add_enabled(false, egui::Checkbox::without_text(&mut dummy));
//                         },
//                     );
//                 });
//                 row.col(|ui| {
//                     ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
//                         ui.add_enabled_ui(false, |ui| {
//                             ui.add_sized(btn_size, egui::Button::new("✖"));
//                         });
//                     });
//                 });
//                 row.col(|ui| {
//                     ui.style_mut().interaction.selectable_labels = false;
//                     ui.label("Base Layer");
//                 });
//                 // row.col(|ui| {
//                 //     ui.add_enabled_ui(false, |ui| {
//                 //         ui.add_sized(btn_size, egui::Button::new("⬆"));
//                 //     });
//                 //     ui.add_enabled_ui(false, |ui| {
//                 //         ui.add_sized(btn_size, egui::Button::new("⬇"));
//                 //     });
//                 // });
//                 if row.response().hovered() {
//                     pointing_hand = true;
//                 }
//                 if row.response().clicked() {
//                     layer_select = LayerSelect::Base;
//                 }
//             });
//             // let profiles_len = args.config.profiles.len();
//             // let mut to_swap: Option<(usize, usize)> = None;
//             for (i, layer) in args
//                 .get_open_profile_ui_mut()
//                 .unwrap()
//                 .layers
//                 .iter_mut()
//                 .enumerate()
//             {
//                 // let first = i == 0;
//                 // let last = i == profiles_len - 1;
//                 body.row(row_height, |mut row| {
//                     row.col(|ui| {
//                         ui.with_layout(
//                             egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
//                             |ui| {
//                                 ui.add(egui::Checkbox::without_text(&mut layer.enabled));
//                             },
//                         );
//                     });
//                     row.col(|ui| {
//                         ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
//                             if ui.add_sized(btn_size, egui::Button::new("✖")).clicked() {
//                                 to_delete = Some(i);
//                             };
//                         });
//                     });
//                     row.col(|ui| {
//                         ui.style_mut().interaction.selectable_labels = false;
//                         ui.label(&layer.name);
//                     });
//                     // row.col(|ui| {
//                     //     ui.add_enabled_ui(!first, |ui| {
//                     //         if ui.add_sized(btn_size, egui::Button::new("⬆")).clicked() {
//                     //             to_swap = Some((i - 1, i));
//                     //         }
//                     //     });
//                     //     ui.add_enabled_ui(!last, |ui| {
//                     //         if ui.add_sized(btn_size, egui::Button::new("⬇")).clicked() {
//                     //             to_swap = Some((i + 1, i));
//                     //         }
//                     //     });
//                     // });
//                     if row.response().hovered() {
//                         pointing_hand = true;
//                     }
//                     if row.response().clicked() {
//                         layer_select = LayerSelect::Other(i);
//                     }
//                 });
//             }
//             // if let Some((a, b)) = to_swap {
//             //     args.config.profiles.swap(a, b);
//             // }
//         });
//     if let Some(to_delete) = to_delete {
//         args.get_open_profile_ui_mut()
//             .unwrap()
//             .layers
//             .remove(to_delete);
//     }
//     match layer_select {
//         LayerSelect::None => (),
//         LayerSelect::Base => {
//             args.gui_local.menu = GuiMenu::ProfileBaseLayerMenu {
//                 profile_idx: args.get_open_profile_ui_idx().unwrap(),
//             };
//         }
//         LayerSelect::Other(i) => {
//             args.gui_local.menu = GuiMenu::ProfileLayerMenu {
//                 profile_idx: args.get_open_profile_ui_idx().unwrap(),
//                 layer_idx: i,
//             }
//         }
//     }
//     if pointing_hand {
//         ui.ctx()
//             .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
//     }
// }
