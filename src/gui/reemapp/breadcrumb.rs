use super::GuiMenu;
use super::ReemApp;

pub fn breadcrumb(ctx: &egui::Context, ui: &mut egui::Ui, args: &mut ReemApp) {
    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
        if let Some(click) = breadcrumb_display(ctx, ui, args) {
            args.gui_local.menu = click;
        }
    });
}

fn breadcrumb_display(_ctx: &egui::Context, ui: &mut egui::Ui, args: &ReemApp) -> Option<GuiMenu> {
    let mut click = None;

    // -------------------- Main Menu Button --------------------

    let main_breadcrumb_response = ui
        .add(egui::Label::new(egui::RichText::new("Reemap").heading()).sense(egui::Sense::click()));
    if main_breadcrumb_response.hovered() {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
    if main_breadcrumb_response.clicked() {
        click = Some(GuiMenu::Main);
    }

    // -------------------- Profile Button --------------------

    if args.gui_local.menu == GuiMenu::Main {
        return click;
    }

    enum ProfileIdx {
        Default,
        Other(usize),
    }
    let profile_idx = match &args.gui_local.menu {
        GuiMenu::Main => unreachable!(),
        GuiMenu::DefaultProfile | GuiMenu::DefaultProfileLayer { .. } => ProfileIdx::Default,
        GuiMenu::Profile { profile_idx } => ProfileIdx::Other(*profile_idx),
        GuiMenu::ProfileLayer {
            profile_idx,
            layer_idx: _,
        } => ProfileIdx::Other(*profile_idx),
    };

    let profile_string = match profile_idx {
        ProfileIdx::Default => String::from("Default Profile"),
        ProfileIdx::Other(profile_idx) => args.config.profiles[profile_idx].name.clone(),
    };

    ui.heading(" > ");

    let profile_breadcrumb_response = ui.add(
        egui::Label::new(egui::RichText::new(&profile_string).heading())
            .sense(egui::Sense::click()),
    );
    if profile_breadcrumb_response.hovered() {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
    if profile_breadcrumb_response.clicked() {
        click = Some(match profile_idx {
            ProfileIdx::Default => GuiMenu::DefaultProfile,
            ProfileIdx::Other(profile_idx) => GuiMenu::Profile { profile_idx },
        });
    }

    // -------------------- Layer Button --------------------

    if matches!(
        args.gui_local.menu,
        GuiMenu::Main | GuiMenu::DefaultProfile | GuiMenu::Profile { .. }
    ) {
        return click;
    }

    let layer_idx = match &args.gui_local.menu {
        GuiMenu::Main | GuiMenu::DefaultProfile | GuiMenu::Profile { .. } => {
            unreachable!()
        }
        GuiMenu::DefaultProfileLayer { layer_idx } => *layer_idx,
        GuiMenu::ProfileLayer {
            profile_idx: _,
            layer_idx,
        } => *layer_idx,
    };

    let layer_string = match profile_idx {
        ProfileIdx::Default => args.config.default.layers[layer_idx].name.clone(),
        ProfileIdx::Other(profile_idx) => args.config.profiles[profile_idx].layers[layer_idx]
            .name
            .clone(),
    };

    ui.heading(" > ");

    let layer_breadcrumb_response = ui.add(
        egui::Label::new(egui::RichText::new(&layer_string).heading()).sense(egui::Sense::click()),
    );
    if layer_breadcrumb_response.hovered() {
        ui.ctx()
            .output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
    if layer_breadcrumb_response.clicked() {
        click = Some(match profile_idx {
            ProfileIdx::Default => GuiMenu::DefaultProfileLayer { layer_idx },
            ProfileIdx::Other(profile_idx) => GuiMenu::ProfileLayer {
                profile_idx,
                layer_idx,
            },
        });
    }
    click
}
