pub fn set_reemap_style(ui: &mut egui::Ui) {
    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
    ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke {
        width: 1.0,
        color: egui::Color32::DARK_GRAY,
    };
    ui.style_mut().visuals.widgets.inactive.fg_stroke = egui::Stroke {
        width: 1.0,
        color: egui::Color32::WHITE,
    };

    ui.style_mut().visuals.widgets.active.weak_bg_fill = egui::Color32::from_black_alpha(100);
    ui.style_mut().visuals.widgets.open.weak_bg_fill = egui::Color32::from_black_alpha(100);

    ui.style_mut().visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_white_alpha(16);
}

pub const REEMAP_SHADOW: egui::Shadow = egui::Shadow {
    offset: [0, 0],
    blur: 16,
    spread: 8,
    color: egui::Color32::from_black_alpha(128),
};

pub const MODAL_BACKDROP_COLOR: egui::Color32 = egui::Color32::from_black_alpha(200);
pub const MODAL_FRAME: egui::Frame = egui::Frame {
    inner_margin: egui::Margin::same(4),
    fill: egui::Color32::from_black_alpha(128),
    stroke: egui::Stroke {
        width: 1.0,
        color: egui::Color32::DARK_GRAY,
    },
    corner_radius: egui::CornerRadius::same(4),
    outer_margin: egui::Margin::ZERO,
    shadow: egui::Shadow::NONE,
};
