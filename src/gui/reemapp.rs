use crate::hooks;

// Thought the name was clever. Don't get too mad, please.
pub struct ReemApp {
    pub text: String,
    pub hookthread_proxy: hooks::HookthreadProxy,
}

impl crate::gui::TrayApp for ReemApp {
    fn update(&mut self, egui_ctx: &egui::Context) {
        catppuccin_egui::set_theme(egui_ctx, catppuccin_egui::MACCHIATO);
        egui::CentralPanel::default().show(egui_ctx, |ui| {
            ui.heading("Hello World!");
            if ui.button("Send text").clicked() {
                println!("Works!");
                self.text.push_str(" More!");
            }
            ui.label(format!("{}", self.text));
        });
    }
}
