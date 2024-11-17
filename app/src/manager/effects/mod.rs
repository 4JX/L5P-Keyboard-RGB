use default_ui::{show_brightness, show_direction};
use eframe::egui::{self, Slider};

use crate::{enums::Effects, profile::Profile};

pub mod ambient;
pub mod christmas;
pub mod default_ui;
pub mod disco;
pub mod fade;
pub mod lightning;
pub mod ripple;
pub mod swipe;
pub mod temperature;
pub mod zones;

impl Effects {
    pub fn show_ui(&mut self, ui: &mut egui::Ui, profile: &mut Profile, update_lights: &mut bool, theme: &crate::gui::style::Theme) {
        match self {
            Effects::AmbientLight { fps, saturation_boost } => {
                ui.scope(|ui| {
                    ui.style_mut().spacing.item_spacing = theme.spacing.default;

                    show_brightness(ui, profile, update_lights);
                    show_direction(ui, profile, update_lights);

                    ui.horizontal(|ui| {
                        *update_lights |= ui.add(Slider::new(fps, 1..=60)).changed();
                        ui.label("FPS");
                    });
                    ui.horizontal(|ui| {
                        *update_lights |= ui.add(Slider::new(saturation_boost, 0.0..=1.0)).changed();
                        ui.label("Saturation Boost");
                    });
                });
            }
            _ => {
                default_ui::show(ui, profile, update_lights, &theme.spacing);
            }
        }
    }
}
