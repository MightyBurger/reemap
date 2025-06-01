#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gui;
mod hooks;

fn main() {
    std::thread::scope(|s| {
        s.spawn(|| {
            hooks::run();
        });
        gui::run();
    });
}
