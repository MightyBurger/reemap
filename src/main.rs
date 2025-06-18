#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod buttons;
mod config;
mod gui;
mod hooks;

fn main() {
    // Reminder: all threads are joined at the end of a std::thread::scope
    std::thread::scope(|s| {
        // Start the hook thread. It will be spawned as a separate thread.
        let hookthread_proxy = hooks::spawn_scoped(s);

        // Run the GUI. It will be ran on this thread, the main thread.
        let app = gui::reemapp::ReemApp {
            text: String::new(),
            hookthread_proxy,
            config: gui::reemapp::ConfigUI::default(),
        };
        gui::run(app);

        // At this point, the GUI closed and is done running.
        // We should close Reemap, so let's stop the hookthread.
        hookthread_proxy.quit();
    });
}
