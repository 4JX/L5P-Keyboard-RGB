use default_ui::{show_brightness, show_direction, show_effect_settings};
use eframe::egui::{self, ComboBox, Slider};
use strum::IntoEnumIterator;

use crate::{
    enums::{Effects, SwipeMode},
    manager::profile::Profile,
};

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

pub fn show_effect_ui(ui: &mut egui::Ui, profile: &mut Profile, update_lights: &mut bool, theme: &crate::gui::style::Theme) {
    let mut effect = profile.effect.clone();

    match &mut effect {
        Effects::SmoothWave { mode, clean_with_black } | Effects::Swipe { mode, clean_with_black } => {
            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = theme.spacing.default;

                show_brightness(ui, profile, update_lights);
                show_direction(ui, profile, update_lights);
                show_effect_settings(ui, profile, update_lights);
                ComboBox::from_label("Swipe mode").width(30.0).selected_text(format!("{:?}", mode)).show_ui(ui, |ui| {
                    for swipe_mode in SwipeMode::iter() {
                        *update_lights |= ui.selectable_value(mode, swipe_mode, format!("{:?}", swipe_mode)).changed();
                    }
                });
                *update_lights |= ui.add_enabled(matches!(mode, SwipeMode::Fill), egui::Checkbox::new(clean_with_black, "Clean with black")).changed();
            });
        }
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

    profile.effect = effect;
}
