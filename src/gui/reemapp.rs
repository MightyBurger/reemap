// Thought the name was clever. Don't get too mad, please.

#[derive(Default)]
pub struct ReemApp {
    text: String,
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
