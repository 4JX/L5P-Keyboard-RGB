use eframe::{
	egui::{CentralPanel, ComboBox, Slider},
	epaint::Vec2,
	CreationContext,
};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, IntoStaticStr};

use crate::{
	effects::EffectManager,
	enums::{Direction, Effects},
	profile::Profile,
};

pub struct App {
	manager: EffectManager,
	profile: Profile,
	selected_brightness: Brightness,
}

#[derive(PartialEq, EnumIter, IntoStaticStr, Clone, Copy)]
enum Brightness {
	Low,
	High,
}

impl App {
	pub fn new(_cc: &CreationContext, manager: EffectManager) -> Self {
		Self {
			manager,
			profile: Profile::default(),
			selected_brightness: Brightness::Low,
		}
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
		CentralPanel::default().show(ctx, |ui| {
			let mut changed = false;

			ui.horizontal(|ui| {
				ui.set_enabled(self.profile.effect.takes_color_array());

				ui.style_mut().spacing.interact_size = Vec2::splat(50.0);

				for i in 0..4 {
					changed |= ui.color_edit_button_srgb(&mut self.profile.rgb_zones[i].rgb).changed();
				}
			});

			ComboBox::from_label("Brightness")
				.selected_text(format! {"{}", {
						let text: &'static str = self.selected_brightness.into();
						text
				}})
				.show_ui(ui, |ui| {
					for val in Brightness::iter() {
						let text: &'static str = val.into();
						changed |= ui.selectable_value(&mut self.selected_brightness, val, text).changed();
					}
				});

			ComboBox::from_label("Effect")
				.selected_text(format! {"{}", {
						let text: &'static str = self.profile.effect.into();
						text
				}})
				.show_ui(ui, |ui| {
					for val in Effects::iter() {
						let text: &'static str = val.into();
						changed |= ui.selectable_value(&mut self.profile.effect, val, text).changed();
					}
				});

			ComboBox::from_label("Direction")
				.selected_text(format! {"{}", {
						let text: &'static str = self.profile.direction.into();
						text
				}})
				.show_ui(ui, |ui| {
					for val in Direction::iter() {
						let text: &'static str = val.into();
						changed |= ui.selectable_value(&mut self.profile.direction, val, text).changed();
					}
				});

			let range = if self.profile.effect.is_built_in() { 1..=3 } else { 1..=10 };
			changed |= ui.add_enabled(self.profile.effect.takes_speed(), Slider::new(&mut self.profile.speed, range)).changed();

			if changed {
				self.profile.brightness = match self.selected_brightness {
					Brightness::Low => 1,
					Brightness::High => 2,
				};

				self.manager.set_profile(self.profile);
			}
		});
	}
}
