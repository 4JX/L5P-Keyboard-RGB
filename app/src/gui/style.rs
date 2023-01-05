use eframe::epaint::Vec2;

pub struct SpacingStyle {
	pub default: Vec2,
	pub large: f32,
	pub medium: f32,
}

impl Default for SpacingStyle {
	fn default() -> Self {
		Self {
			default: Vec2::new(8.0, 5.0),
			large: 20.0,
			medium: 8.0,
		}
	}
}
