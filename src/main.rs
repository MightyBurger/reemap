// Copyright 2025 Jordan Johnson
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod buttons;
mod config;
mod gui;
mod hooks;
mod query_windows;
mod registry;
mod unique;

use clap::Parser;
use etcetera::BaseStrategy;
use tracing::{error, info, instrument, warn};

use crate::gui::ReemapGuiEvent;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(long, hide = true, action)]
    uninstall: bool,

    #[clap(long, short, action, help = "Start minimized to the tray")]
    background: bool,
}

#[instrument]
fn main() {
    tracing_subscriber::fmt::init();

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

    fn display_error_no_ctx(text: &str) {
        native_dialog::DialogBuilder::message()
            .set_level(native_dialog::MessageLevel::Error)
            .set_title("Error opening Reemap")
            .set_text(text)
            .alert()
            .show()
            .unwrap();
        error!("error opening Reemap: {}", text);
    }

    /*
        Initialization sequence:

        1.  Parse command-line arguments.

        2.  Check if called with the --uninstall flag. This is a special case where Reemap removes
            itself from the run-on-login entries.

        3.  Check this is the only running instance of Reemap.

        4.  See if %APPDATA%\Reemap exists. It usually should, except on first launch.
            If it doesn't exist, create the directory.
            Now, %APPDATA%\Reemap certainly exists.

        5.  See if %APPDATA%\Reemap\config.ron exists. It usually should, except on first launch.
            If it doesn't exist, initialize it with a brand new (default) configuration.
            Now, %APPDATA%\Reemap\config.ron certainly exists.

        6.  Read %APPDATA%\Reemap\config.ron and try to parse it into a VersionedConfig struct.
            If that fails, the configuration is probably corrupted. Rather than silently resetting
            it, which may be a destructive action, offer the user the option to reset the
            configuration.

        7.  Convert VersionedConfig to ConfigUI. We want two copies: one to give to the hookthread,
            and one to give to the UI thread.
    */

    let args = Args::parse();

    if args.uninstall {
        info!(
            "called with uninstall flag. NOTE: this feature is intended only to be used by the uninstaller."
        );
        info!("removing entry to start Reemap on login, if such an entry exists");
        let result = registry::unregister_run_on_login();
        if let Err(e) = result {
            error!("could not access registry: {e}");
        }
        return;
    }

    // Ensure this is the only running instance
    let unique_guard = match unique::UniqueGuard::try_lock() {
        Ok(guard) => guard,
        Err(_e) => {
            display_error_no_ctx("Reemap is already running.\n\nIs it hiding in the tray?");
            return;
        }
    };

    // Check %APPDATA%\Reemap
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

    // Check %APPDATA%\Reemap\config.ron
    let config_path = reemap_dir.join("config.ron");
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

    // Read %APPDATA%\Reemap\config.ron
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
                use a configuration file created in a newer version of the software.\n\n\
                Press Yes to continue with the default configuration. Reemap will overwrite the \
                old configuration the next time you click \"Apply\".\n\n\
                Press No to cancel opening the software.",
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

    // Update to latest version
    let config = config::Config::from(versioned_config);

    let start_visible = !args.background;

    // Reminder: all threads are joined at the end of a std::thread::scope
    std::thread::scope(|s| {
        // Build the event loop so we can grab a proxy to the UI thread.
        let event_loop = winit::event_loop::EventLoop::<ReemapGuiEvent>::with_user_event()
            .build()
            .unwrap();
        let ui_proxy = event_loop.create_proxy();

        // Then run the hook thread, giving the UI thread proxy and also getting a proxy to the
        // hookthread at the same time.
        let hookthread_proxy = hooks::spawn_scoped(s, config.clone(), ui_proxy);

        // Run the GUI. It will be ran on this thread, the main thread.
        let app = gui::reemapp::ReemApp::new(hookthread_proxy.clone(), config, config_path);
        gui::run(app, event_loop, start_visible);

        // At this point, the GUI closed and is done running.
        // We should close Reemap, so let's stop the hookthread.
        hookthread_proxy.quit();
    });

    drop(unique_guard);
}
