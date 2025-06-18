use crate::buttons;
use crate::config;
use crate::hooks;

use enum_map::EnumMap;

// Thought the name was clever. Don't get too mad, please.
pub struct ReemApp {
    pub text: String,
    pub hookthread_proxy: hooks::HookthreadProxy,
    pub config: ConfigUI,
}

// Like config::Config, but with extra fields:
//  enabled
//  name
// and without these fields:
//  active
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayerUI {
    // Checked in the UI?
    pub enabled: bool,
    pub layer_type: config::LayerType,
    pub condition: Vec<buttons::HoldButton>,
    pub policy: EnumMap<buttons::Button, config::RemapPolicy>,
    pub name: String,
}

impl From<LayerUI> for config::Layer {
    fn from(value: LayerUI) -> Self {
        Self {
            active: false,
            layer_type: value.layer_type,
            condition: value.condition,
            policy: value.policy,
        }
    }
}

// Like config::Profile, but with extra fields:
//  name
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProfileUI {
    pub base: config::BaseLayer,
    pub layers: Vec<LayerUI>,
    pub name: String,
}

impl From<ProfileUI> for config::Profile {
    fn from(value: ProfileUI) -> Self {
        Self {
            base: value.base,
            layers: value
                .layers
                .into_iter()
                .map(|layer_ui| layer_ui.into())
                .collect(),
        }
    }
}

// Like config::Config, but instantiates ProfileUI instead of Profile and without active_profile
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConfigUI {
    pub default: config::Profile,
    pub profiles: Vec<ProfileUI>,
    pub profile_conditions: Vec<config::ProfileCondition>,
}

impl From<ConfigUI> for config::Config {
    fn from(value: ConfigUI) -> Self {
        Self {
            default: value.default,
            profiles: value
                .profiles
                .into_iter()
                .map(|profiles_ui| profiles_ui.into())
                .collect(),
            profile_conditions: value.profile_conditions,
            active_profile: None,
        }
    }
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
