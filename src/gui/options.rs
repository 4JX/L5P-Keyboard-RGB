use crate::{
	effects::StopSignals,
	enums::{Effects, Message},
};

use super::enums::Colors;
use fltk::{
	enums::{CallbackTrigger, Color, FrameType},
	group::Tile,
	input::IntInput,
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

struct InputChoice;

impl InputChoice {
	fn create(x: i32, y: i32, width: i32, height: i32, title: &str) -> IntInput {
		let mut choice = IntInput::new(x, y, width, height, "").with_label(title);

		choice.set_frame(FrameType::RoundedBox);
		choice.set_color(Color::from_u32(Colors::DarkerGray as u32));
		choice.set_label_color(Color::from_u32(Colors::White as u32));
		choice.set_selection_color(Color::White);
		choice.set_text_color(Color::from_u32(Colors::White as u32));
		choice.set_text_size(20);
		choice.set_label_size(18);
		choice.set_value(1.to_string().as_str());
		choice
	}
}

#[derive(Clone)]
pub struct OptionsTile {
	pub speed_input: IntInput,
	pub brightness_choice: Choice,
	pub direction_choice: Choice,
	pub fps_input: IntInput,
}

impl OptionsTile {
	pub fn create(x: i32, y: i32, tx: &flume::Sender<Message>, stop_signals: &StopSignals) -> Self {
		let mut options_tile = Tile::new(x, y, 1140, 90, "");

		let mut speed_choice = InputChoice::create(x + 25 + 80, y + 25, 45, 35, "Speed: ").center_y(&options_tile);

		let mut brightness_choice = OptionsChoice::create(x + 25 + 140, y + 25, 45, 35, "Brightness: ", "1|2").right_of(&speed_choice, 150);

		let mut direction_choice = OptionsChoice::create(x + 25 + 300, y + 25, 90, 35, "Direction: ", "Left|Right").right_of(&brightness_choice, 130);

		let mut fps_input = InputChoice::create(x + 25 + 80, y + 25, 45, 35, "FPS: ").right_of(&direction_choice, 80);

		options_tile.end();

		options_tile.set_frame(FrameType::FlatBox);
		options_tile.set_color(Color::from_u32(Colors::DarkGray as u32));

		speed_choice.set_trigger(CallbackTrigger::Changed);
		speed_choice.set_maximum_size(2);
		speed_choice.set_callback({
			let tx = tx.clone();
			let stop_signals = stop_signals.clone();
			move |speed_input| {
				stop_signals.store_true();
				if let Ok(speed) = speed_input.value().parse::<u8>() {
					if speed > 5 {
						speed_input.set_value("5");
					} else if speed < 1 {
						speed_input.set_value("1");
					} else {
						speed_input.set_value(&speed.to_string());
					}

					if (1..=5).contains(&speed) {
						tx.send(Message::Refresh).unwrap();
					}
				}
			}
		});

		fps_input.set_trigger(CallbackTrigger::Changed);
		fps_input.set_maximum_size(2);
		fps_input.set_callback({
			let tx = tx.clone();
			let stop_signals = stop_signals.clone();
			move |fps_input| {
				stop_signals.store_true();
				let fps = fps_input.value().parse::<u8>().unwrap_or(1);
				if fps > 60 {
					fps_input.set_value("60");
				} else if fps < 1 {
					fps_input.set_value("1");
				}

				tx.send(Message::Refresh).unwrap();
			}
		});
		fps_input.set_value(&10.to_string());
		// Hide by default
		fps_input.hide();

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
			speed_input: speed_choice,
			brightness_choice,
			direction_choice,
			fps_input,
		}
	}

	pub fn update(&mut self, effect: Effects) {
		//Conditionally activate the direction setting
		if effect.takes_direction() {
			self.direction_choice.activate();
		} else {
			self.direction_choice.deactivate();
		}

		if effect.takes_speed() {
			self.speed_input.activate()
		} else {
			self.speed_input.deactivate()
		}

		if matches!(effect, Effects::AmbientLight { .. }) {
			self.fps_input.show();
		} else {
			self.fps_input.hide();
		}
	}
}
