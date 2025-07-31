use super::GuiMenu;
use super::ReemApp;
use crate::config;
use crate::gui::reemapp::style;
use tracing::error;

pub fn breadcrumb(ctx: &egui::Context, ui: &mut egui::Ui, args: &mut ReemApp) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
        if ui
            .add_sized(style::BUTTON_SIZE, egui::Button::new("Apply"))
            .clicked()
        {
            // Two things happen on Apply.
            // 1. UI therad saves configuration to %APPDATA%
            // 2. UI thread sends config over to hookthread to update the remaps

            let config_str = ron::ser::to_string_pretty(
                &config::VersionedConfig::from(args.config.clone()),
                ron::ser::PrettyConfig::new(),
            )
            .unwrap();
            match std::fs::write(&args.config_path, config_str) {
                Ok(()) => (),
                Err(e) => {
                    error!("could not write to config file: {e}");
                    native_dialog::DialogBuilder::message()
                        .set_level(native_dialog::MessageLevel::Error)
                        .set_title("Error writing file")
                        .set_text(format!(
                            "Reemap could not write to the configuration file.\n\n\
                The applied remaps will take effect, but they will not be saved.\n\n\
                {e}"
                        ))
                        .alert()
                        .show()
                        .unwrap();
                }
            }

            args.hookthread_proxy.update(args.config.clone());
        }

        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            if let Some(click) = breadcrumb_display(ctx, ui, args) {
                args.gui_local.menu = click;
            }
        });
    });
}

fn breadcrumb_display(_ctx: &egui::Context, ui: &mut egui::Ui, args: &ReemApp) -> Option<GuiMenu> {
    let mut click = None;

    // -------------------- Main Menu Button --------------------

    let main_breadcrumb_response =
        ui.add(egui::Label::new(egui::RichText::new("Home").heading()).sense(egui::Sense::click()));
    if main_breadcrumb_response.hovered() {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
    if main_breadcrumb_response.clicked() {
        click = Some(GuiMenu::Main);
    }
    if args.gui_local.menu == GuiMenu::Main {
        return click;
    }

    // -------------------- Profile Button --------------------

    let profile_idx = match &args.gui_local.menu {
        GuiMenu::Main => unreachable!(),
        GuiMenu::Profile { profile_idx } => *profile_idx,
        GuiMenu::ProfileLayer {
            profile_idx,
            layer_idx: _,
        } => *profile_idx,
    };
    let profile_string = &args.config.profiles[profile_idx].name;
    ui.heading(" > ");
    let profile_breadcrumb_response = ui.add(
        egui::Label::new(egui::RichText::new(profile_string).heading())
            .truncate()
            .sense(egui::Sense::click()),
    );
    if profile_breadcrumb_response.hovered() {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
    if profile_breadcrumb_response.clicked() {
        click = Some(GuiMenu::Profile { profile_idx });
    }
    if matches!(args.gui_local.menu, GuiMenu::Main | GuiMenu::Profile { .. }) {
        return click;
    }

    // -------------------- Layer Button --------------------

    let layer_idx = match &args.gui_local.menu {
        GuiMenu::Main | GuiMenu::Profile { .. } => {
            unreachable!()
        }
        GuiMenu::ProfileLayer {
            profile_idx: _,
            layer_idx,
        } => *layer_idx,
    };

    let layer_string = &args.config.profiles[profile_idx].layers[layer_idx].name;

    ui.heading(" > ");

    let layer_breadcrumb_response = ui.add(
        egui::Label::new(egui::RichText::new(layer_string).heading())
            .truncate()
            .sense(egui::Sense::click()),
    );
    if layer_breadcrumb_response.hovered() {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
    if layer_breadcrumb_response.clicked() {
        click = Some(GuiMenu::ProfileLayer {
            profile_idx,
            layer_idx,
        });
    }
    click
}
