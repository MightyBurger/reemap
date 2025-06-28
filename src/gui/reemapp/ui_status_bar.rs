use crate::config;
use crate::settings;

pub fn ui_status_bar(_ctx: &egui::Context, ui: &mut egui::Ui, app: &mut super::ReemApp) {
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
            // Two things happen on Apply.
            // 1. UI therad saves configuration to %APPDATA%
            // 2. UI thread sends config over to hookthread to update the remaps

            let config_str = ron::ser::to_string_pretty(
                &config::VersionedConfig::from(app.config.clone()),
                ron::ser::PrettyConfig::new(),
            )
            .unwrap();
            match std::fs::write(&app.config_path, config_str) {
                Ok(()) => (),
                Err(e) => {
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

            app.hookthread_proxy
                .update(settings::Settings::from(app.config.clone()));
        }
    });
}
