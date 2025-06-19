use crate::buttons;
use crate::config;
use crate::hooks;

use enum_map::EnumMap;

// Thought the name was clever. Don't get too mad, please.
pub struct ReemApp {
    pub hookthread_proxy: hooks::HookthreadProxy,
    pub config: ConfigUI,
    pub gui_local: GuiLocal,
}

pub struct GuiLocal {
    new_profile_modal_open: bool,
    new_profile: ProfileUI,
}

impl Default for GuiLocal {
    fn default() -> Self {
        Self {
            new_profile_modal_open: false,
            new_profile: ProfileUI::default(),
        }
    }
}

// Like config::Config, but with extra fields:
//  enabled
//  name
// and without these fields:
//  active
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayerUI {
    // Checked in the UI?
    pub enabled: bool,
    pub layer_type: config::LayerType,
    pub condition: Vec<buttons::HoldButton>,
    pub policy: EnumMap<buttons::Button, config::RemapPolicy>,
    pub name: String,
}

impl From<LayerUI> for config::Layer {
    fn from(value: LayerUI) -> Self {
        Self {
            active: false,
            layer_type: value.layer_type,
            condition: value.condition,
            policy: value.policy,
        }
    }
}

// Like config::Profile, but with extra fields:
// enabled
//  name
//  condition
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProfileUI {
    pub enabled: bool,
    pub condition: config::ProfileCondition,
    pub base: config::BaseLayer,
    pub layers: Vec<LayerUI>,
    pub name: String,
}

impl Default for ProfileUI {
    fn default() -> Self {
        Self {
            enabled: true,
            condition: config::ProfileCondition::OriBF,
            base: config::BaseLayer::default(),
            layers: Vec::new(),
            name: String::from("New Profile"),
        }
    }
}

impl From<ProfileUI> for config::Profile {
    fn from(value: ProfileUI) -> Self {
        Self {
            base: value.base,
            layers: value
                .layers
                .into_iter()
                .map(|layer_ui| layer_ui.into())
                .collect(),
        }
    }
}

// Like config::Config, but instantiates ProfileUI instead of Profile and without active_profile
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConfigUI {
    pub default: config::Profile,
    pub profiles: Vec<ProfileUI>,
}

impl From<ConfigUI> for config::Config {
    fn from(value: ConfigUI) -> Self {
        Self {
            default: value.default,
            profiles: value
                .profiles
                .iter()
                .cloned()
                .map(|profiles_ui| profiles_ui.into())
                .collect(),
            profile_conditions: value
                .profiles
                .into_iter()
                .map(|profiles_ui| profiles_ui.condition)
                .collect(),
            active_profile: None,
        }
    }
}

impl crate::gui::TrayApp for ReemApp {
    fn update(&mut self, egui_ctx: &egui::Context) {
        // catppuccin_egui::set_theme(egui_ctx, catppuccin_egui::MACCHIATO);
        egui::TopBottomPanel::bottom("Bottom Panel").show(egui_ctx, |ui| {
            ui.horizontal(|ui| {
                egui::Frame::new().inner_margin(2.0).show(ui, |ui| {
                    let left_to_right = egui::Layout {
                        main_dir: egui::Direction::LeftToRight,
                        main_wrap: false,
                        main_align: egui::Align::Min,
                        main_justify: false,
                        cross_align: egui::Align::Center,
                        cross_justify: false,
                    };
                    let right_to_left = egui::Layout {
                        main_dir: egui::Direction::RightToLeft,
                        main_wrap: false,
                        main_align: egui::Align::Min,
                        main_justify: false,
                        cross_align: egui::Align::Center,
                        cross_justify: false,
                    };
                    ui.with_layout(left_to_right, |ui| {
                        ui.label("Reemap");
                    });
                    ui.with_layout(right_to_left, |ui| {
                        if ui.button("Apply").clicked() {
                            self.hookthread_proxy
                                .update(config::Config::from(self.config.clone()));
                        }
                    });
                });
            });
        });
        egui::CentralPanel::default().show(egui_ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                if ui.button("Add Profile").clicked() {
                    self.gui_local.new_profile_modal_open = true;
                }
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    /*
                        Profiles table
                    */
                    egui::Frame::new()
                        // .fill(egui::Color32::RED)
                        .stroke(egui::Stroke {
                            width: 1.0,
                            color: egui::Color32::DARK_GRAY,
                        })
                        .inner_margin(4.0)
                        .corner_radius(4.0)
                        .show(ui, |ui| {
                            profiles_table_ui(ui, self);
                        });

                    if self.gui_local.new_profile_modal_open {
                        let mut ok = false;
                        let mut cancel = false;
                        let modal = egui::Modal::new(egui::Id::new("New Profile Modal")).show(
                            egui_ctx,
                            |ui| {
                                ui.with_layout(
                                    egui::Layout::top_down_justified(egui::Align::LEFT),
                                    |ui| {
                                        ui.label("Profile Name");
                                        ui.text_edit_singleline(
                                            &mut self.gui_local.new_profile.name,
                                        );
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.button("Cancel").clicked() {
                                                    cancel = true;
                                                }
                                                if ui.button("OK").clicked() {
                                                    ok = true;
                                                }
                                            },
                                        );
                                    },
                                );
                            },
                        );
                        if egui_ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                            ok = true;
                        }
                        if modal.should_close() {
                            cancel = true;
                        }
                        if ok {
                            self.config
                                .profiles
                                .push(self.gui_local.new_profile.clone());
                            self.gui_local.new_profile_modal_open = false;
                        } else if cancel {
                            self.gui_local.new_profile_modal_open = false;
                        }
                    }
                });
            });
        });
    }
}

fn profiles_table_ui(ui: &mut egui::Ui, args: &mut ReemApp) {
    use egui_extras::{Column, TableBuilder};
    let header_height = 12.0;
    let row_height = 20.0;
    let btn_size = [20.0, 20.0];
    TableBuilder::new(ui)
        .striped(true)
        .auto_shrink(false)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(60.0))
        .column(Column::remainder())
        // .column(Column::exact(60.0))
        .header(header_height, |mut header| {
            header.col(|ui| {
                ui.strong("Enabled");
            });
            header.col(|ui| {
                ui.strong("Name");
            });
        })
        .body(|mut body| {
            body.row(row_height, |mut row| {
                let mut dummy = true;
                row.col(|ui| {
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            ui.add_enabled(false, egui::Checkbox::without_text(&mut dummy));
                        },
                    );
                });
                row.col(|ui| {
                    ui.label("Default");
                });
                // row.col(|ui| {
                //     ui.add_enabled_ui(false, |ui| {
                //         ui.add_sized(btn_size, egui::Button::new("⬆"));
                //     });
                //     ui.add_enabled_ui(false, |ui| {
                //         ui.add_sized(btn_size, egui::Button::new("⬇"));
                //     });
                // });
            });
            let profiles_len = args.config.profiles.len();
            let mut to_swap: Option<(usize, usize)> = None;
            for (i, profile) in args.config.profiles.iter_mut().enumerate() {
                let first = i == 0;
                let last = i == profiles_len - 1;
                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.with_layout(
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                ui.add(egui::Checkbox::without_text(&mut profile.enabled));
                            },
                        );
                    });
                    row.col(|ui| {
                        ui.label(&profile.name);
                    });
                    // row.col(|ui| {
                    //     ui.add_enabled_ui(!first, |ui| {
                    //         if ui.add_sized(btn_size, egui::Button::new("⬆")).clicked() {
                    //             to_swap = Some((i - 1, i));
                    //         }
                    //     });
                    //     ui.add_enabled_ui(!last, |ui| {
                    //         if ui.add_sized(btn_size, egui::Button::new("⬇")).clicked() {
                    //             to_swap = Some((i + 1, i));
                    //         }
                    //     });
                    // });
                });
            }
            if let Some((a, b)) = to_swap {
                args.config.profiles.swap(a, b);
            }
        });
}
// ✖

fn new_profile_modal(ui: &mut egui::Ui, args: &mut ReemApp) {}
