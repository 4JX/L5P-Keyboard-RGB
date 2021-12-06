use crate::{enums::Message, keyboard_manager::StopSignals};

use super::enums::Colors;
use fltk::{
	enums::{Align, Color, FrameType},
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
	pub direction_choice: Choice,
}

impl OptionsTile {
	pub fn create(x: i32, y: i32, tx: flume::Sender<Message>, stop_signals: &StopSignals) -> Self {
		let mut options_tile = Tile::new(x, y, 1140, 90, "");

		let mut speed_choice = OptionsChoice::create(x + 25 + 80, y + 25, 45, 35, "Speed: ", "1|2|3|4").center_y(&options_tile);

		let mut brightness_choice = OptionsChoice::create(x + 25 + 140, y + 25, 45, 35, "Brightness: ", "1|2").right_of(&speed_choice, 150);

		let mut direction_choice = OptionsChoice::create(x + 25 + 300, y + 25, 90, 35, "Direction: ", "Left|Right").right_of(&brightness_choice, 130);

		options_tile.end();

		options_tile.set_frame(FrameType::FlatBox);
		options_tile.set_color(Color::from_u32(Colors::DarkGray as u32));

		speed_choice.set_callback({
			let tx = tx.clone();
			let stop_signals = stop_signals.clone();
			move |choice| {
				stop_signals.store_true();
				if let Some(value) = choice.choice() {
					let speed = value.parse::<u8>().unwrap();
					if (1..=4).contains(&speed) {
						tx.send(Message::Refresh).unwrap();
					}
				}
			}
		});

		brightness_choice.set_callback({
			let tx = tx.clone();
			let stop_signals = stop_signals.clone();
			move |choice| {
				stop_signals.store_true();
				if let Some(value) = choice.choice() {
					let brightness = value.parse::<u8>().unwrap();
					if (1..=2).contains(&brightness) {
						tx.send(Message::Refresh).unwrap();
					}
				}
			}
		});

		direction_choice.set_callback({
			let tx = tx.clone();
			let stop_signals = stop_signals.clone();
			move |choice| {
				stop_signals.store_true();
				if let Some(_value) = choice.choice() {
					tx.send(Message::Refresh).unwrap();
				}
			}
		});

		Self {
			speed_choice,
			brightness_choice,
			direction_choice,
		}
	}
}
