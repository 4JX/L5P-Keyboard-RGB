use super::enums::Colors;
use fltk::{
	enums::{Color, FrameType},
	group::Tile,
	menu::Choice,
	prelude::*,
};

struct OptionsChoice;

impl OptionsChoice {
	fn create(x: i32, y: i32, width: i32, height: i32, title: &str, choices: &str) -> Choice {
		let mut choice = Choice::new(x, y, width, height, "").with_label(title);
		choice.add_choice(choices);

		choice.set_frame(FrameType::RoundedBox);
		choice.set_color(Color::from_u32(Colors::DarkerGray as u32));
		choice.set_label_color(Color::from_u32(Colors::White as u32));
		choice.set_selection_color(Color::White);
		choice.set_text_color(Color::from_u32(Colors::White as u32));
		choice.set_text_size(20);
		choice.set_label_size(18);
		choice.set_value(0);
		choice
	}
}

#[derive(Clone)]
pub struct OptionsTile {
	pub speed_choice: Choice,
	pub brightness_choice: Choice,
}

impl OptionsTile {
	pub fn create(x: i32, y: i32) -> Self {
		let mut options_tile = Tile::new(x, y, 360, 90, "");
		let speed_choice = OptionsChoice::create(x + 100, y + 25, 45, 40, "Speed: ", "1|2|3|4");

		let brightness_choice = OptionsChoice::create(x + 100 + 190, y + 25, 45, 40, "Brightness: ", "1|2");
		options_tile.end();

		options_tile.set_frame(FrameType::FlatBox);
		options_tile.set_color(Color::from_u32(Colors::DarkGray as u32));

		Self { speed_choice, brightness_choice }
	}
}
