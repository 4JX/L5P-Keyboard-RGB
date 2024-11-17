use eframe::egui::{ComboBox, Slider, Ui};
use legion_rgb_driver::SPEED_RANGE;
use strum::IntoEnumIterator;

use crate::{
    enums::{Brightness, Direction},
    gui::style::SpacingStyle,
    profile::Profile,
};

const COMBOBOX_WIDTH: f32 = 20.0;

pub fn show(ui: &mut Ui, profile: &mut Profile, update_lights: &mut bool, spacing: &SpacingStyle) {
    ui.scope(|ui| {
        ui.style_mut().spacing.item_spacing = spacing.default;

        show_brightness(ui, profile, update_lights);
        show_direction(ui, profile, update_lights);
        show_effect_settings(ui, profile, update_lights);
    });
}

pub fn show_brightness(ui: &mut Ui, profile: &mut Profile, update_lights: &mut bool) {
    ComboBox::from_label("Brightness")
        .width(COMBOBOX_WIDTH)
        .selected_text({
            let text: &'static str = profile.brightness.into();
            text
        })
        .show_ui(ui, |ui| {
            for val in Brightness::iter() {
                let text: &'static str = val.into();
                *update_lights |= ui.selectable_value(&mut profile.brightness, val, text).changed();
            }
        });
}

pub fn show_direction(ui: &mut Ui, profile: &mut Profile, update_lights: &mut bool) {
    ui.add_enabled_ui(profile.effect.takes_direction(), |ui| {
        ComboBox::from_label("Direction")
            .width(COMBOBOX_WIDTH)
            .selected_text({
                let text: &'static str = profile.direction.into();
                text
            })
            .show_ui(ui, |ui| {
                for val in Direction::iter() {
                    let text: &'static str = val.into();
                    *update_lights |= ui.selectable_value(&mut profile.direction, val, text).changed();
                }
            });
    });
}

pub fn show_effect_settings(ui: &mut Ui, profile: &mut Profile, update_lights: &mut bool) {
    let range = if profile.effect.is_built_in() { SPEED_RANGE } else { 1..=10 };

    ui.horizontal(|ui| {
        *update_lights |= ui.add_enabled(profile.effect.takes_speed(), Slider::new(&mut profile.speed, range)).changed();
        ui.label("Speed");
    });
}
