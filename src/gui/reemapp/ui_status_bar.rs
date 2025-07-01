use tracing::error;

use crate::config;

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
        // use crate::hooks::hooklocal::{ActiveProfile, HOOKLOCAL};
        // let mut hook_local = HOOKLOCAL.lock().expect("mutex poisoned");
        // let hook_local = hook_local
        //     .as_mut()
        //     .expect("local data should have been initialized");

        // let (current_layers, current_layer_actives): (&[config::Layer], &[bool]) =
        //     match hook_local.active_profile {
        //         ActiveProfile::Default => (
        //             &hook_local.config.default.layers,
        //             &mut hook_local.active_layers_default,
        //         ),
        //         ActiveProfile::Other(profile_idx) => (
        //             &hook_local.config.profiles[profile_idx].layers,
        //             &mut hook_local.active_layers_profile[profile_idx],
        //         ),
        //     };

        // let profile_str = match hook_local.active_profile {
        //     ActiveProfile::Default => "Default Profile".to_string(),
        //     ActiveProfile::Other(profile_idx) => {
        //         hook_local.config.profiles[profile_idx].name.clone()
        //     }
        // };

        // let active_layers_str: String = if current_layer_actives.iter().all(|active| !active) {
        //     String::from("(no active layers)")
        // } else {
        //     itertools::Itertools::intersperse(
        //         current_layers
        //             .iter()
        //             .zip(current_layer_actives)
        //             .filter(|(_, active)| **active)
        //             .map(|(layer, _)| layer.name.clone()),
        //         String::from(", "),
        //     )
        //     .collect()
        // };

        // ui.label(format!("{profile_str} | {active_layers_str}"));
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

            app.hookthread_proxy.update(app.config.clone());
        }
    });
}
