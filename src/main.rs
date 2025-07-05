#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod buttons;
mod config;
mod gui;
mod hooks;

use etcetera::BaseStrategy;
use tracing::{error, info, instrument, warn};

#[instrument]
fn main() {
    tracing_subscriber::fmt::init();
    /*
        Initialization sequence:

        1.  See if %APPDATA%\Reemap exists. It usually should, except on first launch.
            If it doesn't exist, create the directory.
            Now, %APPDATA%\Reemap certainly exists.

        2.  See if %APPDATA%\Reemap\config.ron exists. It usually should, except on first launch.
            If it doesn't exist, initialize it with a brand new (default) configuration.
            Now, %APPDATA%\Reemap\config.ron certainly exists.

        3.  Read %APPDATA%\Reemap\config.ron and try to parse it into a VersionedConfig struct.
            If that fails, the configuration is probably corrupted. Rather than silently resetting
            it, which may be a destructive action, offer the user the option to reset the
            configuration.

        4.  Convert VersionedConfig to ConfigUI. We want two copies: one to give to the hookthread,
            and one to give to the UI thread.
    */

    // Open the configuration file, or create it if it doesn't already exist.

    fn display_error(text: &str, ctx: impl std::fmt::Display) {
        let body_text = format!("{text}\n\n{ctx}");
        native_dialog::DialogBuilder::message()
            .set_level(native_dialog::MessageLevel::Error)
            .set_title("Error opening Reemap")
            .set_text(&body_text)
            .alert()
            .show()
            .unwrap();
        error!("error opening Reemap: {}", &body_text);
    }

    // Step 1
    let mut reemap_dir = etcetera::choose_base_strategy().unwrap().config_dir();
    reemap_dir.push("Reemap");
    let reemap_dir_exists = match reemap_dir.try_exists() {
        Ok(exists) => exists,
        Err(e) => {
            display_error(
                "Reemap could not check whether the configuration directory exists.",
                e,
            );
            return;
        }
    };
    if !reemap_dir_exists {
        match std::fs::create_dir(&reemap_dir) {
            Ok(()) => (),
            Err(e) => {
                display_error("Reemap could not create the configuration directory.", e);
                return;
            }
        }
    }

    // Step 2
    let config_path = reemap_dir.join("remaps.ron");
    let config_file_exists = match config_path.try_exists() {
        Ok(exists) => exists,
        Err(e) => {
            display_error("Reemap could not open the configuration file.", e);
            return;
        }
    };
    if !config_file_exists {
        let default_config = config::VersionedConfig::default();
        let default_config =
            ron::ser::to_string_pretty(&default_config, ron::ser::PrettyConfig::new()).unwrap();
        match std::fs::write(&config_path, default_config) {
            Ok(()) => (),
            Err(e) => {
                display_error("Reemap could not create the configuration file.", e);
                return;
            }
        }
    }

    // Step 3
    let config_str = match std::fs::read_to_string(&config_path) {
        Ok(s) => s,
        Err(e) => {
            display_error("Reemap could not read the configuration file.", e);
            return;
        }
    };

    let versioned_config: config::VersionedConfig = match ron::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => {
            warn!("failed to parse config file: {e}");
            let regenerate = native_dialog::DialogBuilder::message()
                .set_level(native_dialog::MessageLevel::Warning)
                .set_title("Corrupted configuration file")
                .set_text(
                    "Reemap could not parse the configuration file. This may happen if you \
                use a configuration file created in a newer version of Reemap.\n\n\
                Press Yes to continue with the default configuration. The configuration will be \
                overwritten when you click \"Apply\" in Reemap.\n\n\
                Press No to close Reemap.",
                )
                .confirm()
                .show()
                .unwrap();
            if regenerate {
                info!("going forward with the default configuration");
                config::VersionedConfig::default()
            } else {
                info!("exiting now");
                return;
            }
        }
    };

    let config = config::Config::from(versioned_config);

    // Reminder: all threads are joined at the end of a std::thread::scope
    std::thread::scope(|s| {
        // Start the hook thread. It will be spawned as a separate thread.
        let hookthread_proxy = hooks::spawn_scoped(s, config.clone());

        // Run the GUI. It will be ran on this thread, the main thread.
        let app = gui::reemapp::ReemApp {
            hookthread_proxy: hookthread_proxy.clone(),
            config,
            gui_local: gui::reemapp::GuiLocal::default(),
            config_path,
        };
        gui::run(app);

        // At this point, the GUI closed and is done running.
        // We should close Reemap, so let's stop the hookthread.
        hookthread_proxy.quit();
    });
}
