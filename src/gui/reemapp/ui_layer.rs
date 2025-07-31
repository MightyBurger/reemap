use crate::buttons;
use crate::config;
use crate::gui::reemapp::RemapsSearchOpts;
use crate::gui::reemapp::style;
use crate::gui::reemapp::ui_edit_layer_modal::ui_edit_layer_modal;
use crate::gui::reemapp::ui_ok_cancel_modal::ui_ok_cancel_modal;
use crate::gui::reemapp::ui_tables::ui_available_buttons_table;
use crate::gui::reemapp::ui_tables::ui_rearrange_table;
use crate::gui::reemapp::{EditLayerModalOpts, NewRemapModalOpts, RemapPolicyUI};
use smallvec::SmallVec;
use strum::IntoEnumIterator;

pub fn ui_layer(
    ui: &mut egui::Ui,
    layer: &mut config::Layer,
    new_remap_modal: &mut NewRemapModalOpts,
    edit_layer_modal: &mut EditLayerModalOpts,
    remaps_search: &mut RemapsSearchOpts,
    show_rare_keys: bool,
) {
    use crate::gui::reemapp::style::REEMAP_SHADOW;
    use egui_extras::{Size, StripBuilder};

    egui::Frame::new().shadow(REEMAP_SHADOW).show(ui, |ui| {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.label(layer.condition_helper_text());
            let edit_response = ui.add_sized(style::BUTTON_SIZE, egui::Button::new("Edit"));
            if edit_response.clicked() {
                *edit_layer_modal = EditLayerModalOpts {
                    modal_open: true,
                    name: layer.name.clone(),
                    layer_type: layer.layer_type.clone(),
                    condition: layer.condition.clone(),
                    search: String::new(),
                };
            }
            ui.add_space(style::SPACING);
            StripBuilder::new(ui)
                .size(Size::remainder())
                .size(Size::initial(style::BUTTON_HEIGHT))
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        style::UI_FRAME.show(ui, |ui| {
                            ui_remaps_table(
                                ui,
                                layer,
                                new_remap_modal,
                                remaps_search,
                                show_rare_keys,
                            );
                        });
                    });
                    strip.strip(|builder| {
                        builder
                            .size(Size::relative(0.35))
                            .size(Size::remainder())
                            .horizontal(|mut strip| {
                                strip.cell(|ui| {
                                    ui.add(
                                        egui::TextEdit::singleline(
                                            &mut remaps_search.search_string,
                                        )
                                        .hint_text("Search"),
                                    );
                                });
                                strip.cell(|ui| {
                                    ui.add(egui::Checkbox::new(
                                        &mut remaps_search.hide_unmapped,
                                        "Hide deferred",
                                    ));
                                });
                            });
                    });
                });
        });
    });

    // ----- New remap modal -----

    if let Some(button) = new_remap_modal.modal_open {
        let policy = &mut layer.policy[button];
        ui_new_remap_modal(ui, new_remap_modal, button, policy, show_rare_keys);
    }

    // ----- Edit layer modal -----

    if edit_layer_modal.modal_open {
        let ok_cancel = ui_edit_layer_modal(
            ui,
            edit_layer_modal,
            &format!("Editing layer {}", layer.name),
            show_rare_keys,
        );
        match ok_cancel {
            Some(true) => {
                layer.name = edit_layer_modal.name.clone();
                layer.layer_type = edit_layer_modal.layer_type.clone();
                layer.condition = edit_layer_modal.condition.clone();
                edit_layer_modal.modal_open = false;
            }
            Some(false) => {
                edit_layer_modal.modal_open = false;
            }
            None => (),
        }
    }
}

fn ui_remaps_table(
    ui: &mut egui::Ui,
    layer: &mut config::Layer,
    new_remap_modal: &mut NewRemapModalOpts,
    remaps_search: &RemapsSearchOpts,
    show_rare_keys: bool,
) {
    use buttons::Button;
    use buttons::key::KeyType;
    use egui_extras::{Column, TableBuilder};

    let mut pointing_hand = false;
    let mut button_select = None;
    TableBuilder::new(ui)
        .id_salt("Remaps Table")
        .striped(true)
        .auto_shrink(false)
        .sense(egui::Sense::click_and_drag())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(60.0)) // Device
        .column(Column::exact(120.0)) // Button
        .column(Column::remainder()) // Policy
        .header(style::HEADER_HEIGHT, |mut header| {
            header.col(|ui| {
                ui.strong("Device");
            });
            header.col(|ui| {
                ui.strong("Input");
            });
            header.col(|ui| {
                ui.strong("Output");
            });
        })
        .body(|mut body| {
            let key_iter = buttons::key::KeyButton::iter()
                .filter(|key| {
                    match (
                        &layer.policy[buttons::Button::from(*key)],
                        show_rare_keys,
                        key.key_type(),
                    ) {
                        // If a remap exists for a key, show it no matter what.
                        (config::RemapPolicy::NoRemap | config::RemapPolicy::Remap(_), _, _) => {
                            true
                        }
                        // Otherwise, if show_rare_keys is true, show if it's a common or rare key.
                        (_, true, KeyType::Common | KeyType::Rare) => true,
                        // Otherwise, only show if it's a common key.
                        (_, false, KeyType::Common) => true,
                        _ => false,
                    }
                })
                .map(buttons::Button::from);
            let mouse_iter = buttons::mouse::MouseButton::iter().map(buttons::Button::from);
            let wheel_iter = buttons::wheel::MouseWheelButton::iter().map(buttons::Button::from);

            for button in mouse_iter
                .chain(wheel_iter)
                .chain(key_iter)
                .filter(|button| {
                    let mod_search = remaps_search.search_string.trim().to_lowercase();
                    mod_search.is_empty()
                        || button.to_string().to_lowercase().contains(&mod_search)
                        || if let config::RemapPolicy::Remap(ref outputs) = layer.policy[*button]
                            && outputs.iter().any(|output| {
                                output.to_string().to_lowercase().contains(&mod_search)
                            })
                        {
                            true
                        } else {
                            false
                        }
                })
                .filter(|button| {
                    !remaps_search.hide_unmapped
                        || !matches!(layer.policy[*button], config::RemapPolicy::Defer)
                })
            {
                body.row(style::ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        let device = match button {
                            Button::Key(_) => "Keyboard",
                            Button::Mouse(_) | Button::Wheel(_) => "Mouse",
                        };
                        ui.label(device);
                    });
                    row.col(|ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.label(button.to_string());
                    });
                    row.col(|ui| {
                        let policy = layer.policy[button].clone();
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.label(policy.to_string());
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
    if let Some(button) = button_select {
        *new_remap_modal = NewRemapModalOpts {
            modal_open: Some(button),
            policy: match layer.policy[button] {
                config::RemapPolicy::Defer => RemapPolicyUI::Defer,
                config::RemapPolicy::NoRemap => RemapPolicyUI::NoRemap,
                config::RemapPolicy::Remap(_) => RemapPolicyUI::Remap,
            },
            outputs: match layer.policy[button] {
                config::RemapPolicy::Defer | config::RemapPolicy::NoRemap => SmallVec::new(),
                config::RemapPolicy::Remap(ref output) => output.clone(),
            },
            search: String::new(),
        };
    }
    if pointing_hand {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
}

fn ui_new_remap_modal(
    ui: &mut egui::Ui,
    modal_opts: &mut NewRemapModalOpts,
    button: buttons::Button,
    policy: &mut config::RemapPolicy,
    show_rare_keys: bool,
) {
    use egui_extras::{Size, StripBuilder};

    let helper_text = get_new_remap_helper_text(&button, &modal_opts.outputs, &modal_opts.policy);
    let ok_cancel = ui_ok_cancel_modal(ui, &helper_text, true, |ui| {
        ui.heading(format!("Remaps for {button}"));
        ui.separator();
        ui.add_space(style::SPACING);
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            ui.label("Policy");
            egui::ComboBox::from_id_salt("policy")
                .selected_text(format!("{}", &modal_opts.policy))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut modal_opts.policy, RemapPolicyUI::Defer, "Defer");
                    ui.selectable_value(&mut modal_opts.policy, RemapPolicyUI::NoRemap, "No Remap");
                    ui.selectable_value(&mut modal_opts.policy, RemapPolicyUI::Remap, "Remap");
                });
            ui.add_space(style::SPACING);

            let enable_tables = match modal_opts.policy {
                RemapPolicyUI::Defer | RemapPolicyUI::NoRemap => false,
                RemapPolicyUI::Remap => true,
            };
            ui.add_enabled_ui(enable_tables, |ui| {
                ui.columns_const(|[col_1, col_2]| {
                    style::UI_FRAME.show(col_1, |ui| {
                        ui_rearrange_table(ui, &mut modal_opts.outputs, "Output");
                    });
                    StripBuilder::new(col_2)
                        .size(Size::remainder())
                        .size(Size::initial(style::BUTTON_HEIGHT))
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                style::UI_FRAME.show(ui, |ui| {
                                    ui_available_buttons_table(
                                        ui,
                                        &mut modal_opts.outputs,
                                        &modal_opts.search,
                                        show_rare_keys,
                                    );
                                });
                            });
                            strip.cell(|ui| {
                                ui.add_sized(
                                    [ui.available_width(), style::BUTTON_HEIGHT],
                                    egui::TextEdit::singleline(&mut modal_opts.search)
                                        .hint_text("Search"),
                                );
                            });
                        });
                });
            });
        });
    });
    match ok_cancel {
        Some(true) => {
            *policy = match modal_opts.policy {
                RemapPolicyUI::Defer => config::RemapPolicy::Defer,
                RemapPolicyUI::NoRemap => config::RemapPolicy::NoRemap,
                RemapPolicyUI::Remap => config::RemapPolicy::Remap(modal_opts.outputs.clone()),
            };
            modal_opts.modal_open = None;
        }
        Some(false) => {
            modal_opts.modal_open = None;
        }
        None => (),
    }
}

fn get_new_remap_helper_text(
    button: &buttons::Button,
    outputs: &[buttons::Button],
    policy: &RemapPolicyUI,
) -> String {
    match policy {
        RemapPolicyUI::Defer => format!("This layer will not affect {button} inputs."),
        RemapPolicyUI::NoRemap => {
            format!("When active, this layer will prevent {button} from being remapped.")
        }
        RemapPolicyUI::Remap => {
            let buttons_str: String = if outputs.is_empty() {
                String::from("(no inputs)")
            } else {
                itertools::Itertools::intersperse(
                    outputs.iter().map(|btn| btn.to_string()),
                    String::from(", "),
                )
                .collect()
            };
            format!("When active, this layer will remap {button} to the following: {buttons_str}.")
        }
    }
}
