use crate::enums::Colors;
use fltk::{
	enums::{Color, FrameType},
	group::Tile,
	menu::Choice,
	prelude::*,
};

struct OptionsChoice;

impl OptionsChoice {
	fn new(x: i32, y: i32, width: i32, height: i32, title: &str, choices: &str) -> Choice {
		let mut choice = Choice::new(x, y, width, height, "").with_label(title);
		choice.add_choice(choices);

		//Themeing
		choice.set_frame(FrameType::FlatBox);
		choice.set_color(Color::from_u32(Colors::DarkGray as u32));
		choice.set_label_color(Color::from_u32(Colors::White as u32));
		choice.set_selection_color(Color::White);
		choice.set_text_color(Color::from_u32(Colors::White as u32));
		choice.set_text_size(20);
		choice.set_label_size(20);
		choice.set_value(0);
		choice
	}
}
pub struct OptionsTile {
	pub speed_choice: Choice,
	pub brightness_choice: Choice,
}

impl OptionsTile {
	pub fn new() -> Self {
		let mut options_tile = Tile::new(540, 360, 360, 90, "");
		let speed_choice = OptionsChoice::new(540 + 100, 385, 40, 40, "Speed:", "1|2|3|4");

		let brightness_choice = OptionsChoice::new(540 + 100 + 190, 385, 40, 40, "Brightness:", "1|2");
		options_tile.end();

		// Options tile
		options_tile.set_frame(FrameType::FlatBox);

		Self { speed_choice, brightness_choice }
	}
}
