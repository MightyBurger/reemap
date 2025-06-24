use super::GuiMenu;
use super::ReemApp;

pub fn breadcrumb(ctx: &egui::Context, ui: &mut egui::Ui, args: &mut ReemApp) {
    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
        if let Some(click) = breadcrumb_display(ctx, ui, &args) {
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
        click = Some(GuiMenu::MainMenu);
    }

    // -------------------- Profile Button --------------------

    if args.gui_local.menu == GuiMenu::MainMenu {
        return click;
    }

    enum ProfileIdx {
        Default,
        Other(usize),
    }
    let profile_idx = match &args.gui_local.menu {
        GuiMenu::MainMenu => unreachable!(),
        GuiMenu::DefaultProfileMenu
        | GuiMenu::DefaultProfileLayerMenu { .. }
        | GuiMenu::DefaultProfileBaseLayerMenu => ProfileIdx::Default,
        GuiMenu::ProfileMenu { profile_idx } => ProfileIdx::Other(*profile_idx),
        GuiMenu::ProfileBaseLayerMenu { profile_idx } => ProfileIdx::Other(*profile_idx),
        GuiMenu::ProfileLayerMenu {
            profile_idx,
            layer_idx: _,
        } => ProfileIdx::Other(*profile_idx),
    };

    let profile_string = match profile_idx {
        ProfileIdx::Default => String::from("Default Profile"),
        ProfileIdx::Other(profile_idx) => format!("{}", &args.config.profiles[profile_idx].name),
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
            ProfileIdx::Default => GuiMenu::DefaultProfileMenu,
            ProfileIdx::Other(profile_idx) => GuiMenu::ProfileMenu { profile_idx },
        });
    }

    // -------------------- Layer Button --------------------

    if matches!(
        args.gui_local.menu,
        GuiMenu::MainMenu | GuiMenu::DefaultProfileMenu | GuiMenu::ProfileMenu { .. }
    ) {
        return click;
    }

    enum LayerIdx {
        Base,
        Other(usize),
    }
    let layer_idx = match &args.gui_local.menu {
        GuiMenu::MainMenu | GuiMenu::DefaultProfileMenu | GuiMenu::ProfileMenu { .. } => {
            unreachable!()
        }
        GuiMenu::DefaultProfileBaseLayerMenu => LayerIdx::Base,
        GuiMenu::DefaultProfileLayerMenu { layer_idx } => LayerIdx::Other(*layer_idx),
        GuiMenu::ProfileBaseLayerMenu { profile_idx: _ } => LayerIdx::Base,
        GuiMenu::ProfileLayerMenu {
            profile_idx: _,
            layer_idx,
        } => LayerIdx::Other(*layer_idx),
    };

    let layer_string = match layer_idx {
        LayerIdx::Base => String::from("Base Layer"),
        LayerIdx::Other(layer_idx) => match profile_idx {
            ProfileIdx::Default => format!("{}", &args.config.default.layers[layer_idx].name),
            ProfileIdx::Other(profile_idx) => format!(
                "{}",
                &args.config.profiles[profile_idx].layers[layer_idx].name
            ),
        },
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
        click = Some(match (profile_idx, layer_idx) {
            (ProfileIdx::Default, LayerIdx::Base) => GuiMenu::DefaultProfileBaseLayerMenu,
            (ProfileIdx::Other(profile_idx), LayerIdx::Base) => {
                GuiMenu::ProfileBaseLayerMenu { profile_idx }
            }
            (ProfileIdx::Default, LayerIdx::Other(layer_idx)) => {
                GuiMenu::DefaultProfileLayerMenu { layer_idx }
            }
            (ProfileIdx::Other(profile_idx), LayerIdx::Other(layer_idx)) => {
                GuiMenu::ProfileLayerMenu {
                    profile_idx,
                    layer_idx,
                }
            }
        });
    }
    click
}
