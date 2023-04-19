use eframe::egui::{ComboBox, Slider, Ui};
use legion_rgb_driver::SPEED_RANGE;
use strum::IntoEnumIterator;

use crate::{
    enums::{Brightness, Direction, Effects},
    profile::Profile,
};

use super::style::SpacingStyle;

#[derive(Default)]
pub struct EffectOptions;

const COMBOBOX_WIDTH: f32 = 20.0;

impl EffectOptions {
    pub fn show(&mut self, ui: &mut Ui, profile: &mut Profile, update_lights: &mut bool, spacing: &SpacingStyle) {
        ui.scope(|ui| {
            ui.style_mut().spacing.item_spacing = spacing.default;

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

            ui.scope(|ui| {
                ui.set_enabled(profile.effect.takes_direction());

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

            if let Effects::AmbientLight { fps } = &mut profile.effect {
                ui.horizontal(|ui| {
                    *update_lights |= ui.add(Slider::new(fps, 1..=60)).changed();
                    ui.label("FPS");
                })
            } else {
                let range = if profile.effect.is_built_in() { SPEED_RANGE } else { 1..=10 };

                ui.horizontal(|ui| {
                    *update_lights |= ui.add_enabled(profile.effect.takes_speed(), Slider::new(&mut profile.speed, range)).changed();
                    ui.label("Speed");
                })
            }
        });
    }
}
