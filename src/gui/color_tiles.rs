use super::enums::{BaseColor, Colors};
use crate::{
	enums::{Effects, Message},
	keyboard_manager::StopSignals,
};
use fltk::{
	button::ToggleButton,
	enums::{Color, Event, FrameType},
	group::{Pack, PackType, Tile},
	input::IntInput,
	prelude::*,
};

const TILE_WIDTH: i32 = 540;
const TILE_HEIGHT: i32 = 90;

pub struct ColorInput;

impl ColorInput {
	pub fn create(x: i32, y: i32, width: i32, height: i32, color: BaseColor, tx: flume::Sender<Message>, stop_signals: StopSignals) -> IntInput {
		let mut color_input = match color {
			BaseColor::Red => {
				let mut color_input = IntInput::new(x, y, width, height, "R:");
				color_input.set_label_color(Color::from_u32(Colors::Red as u32));
				color_input
			}
			BaseColor::Green => {
				let mut color_input = IntInput::new(x, y, width, height, "G:");
				color_input.set_label_color(Color::from_u32(Colors::Green as u32));
				color_input
			}
			BaseColor::Blue => {
				let mut color_input = IntInput::new(x, y, width, height, "B:");
				color_input.set_label_color(Color::from_u32(Colors::Blue as u32));
				color_input
			}
		};
		color_input.set_frame(FrameType::RFlatBox);
		color_input.set_color(Color::from_u32(Colors::DarkGray as u32));
		color_input.set_selection_color(Color::White);
		color_input.set_text_color(Color::from_u32(Colors::White as u32));
		color_input.set_text_size(30);
		color_input.set_label_size(30);
		color_input.set_value("0");
		color_input.set_maximum_size(4);

		color_input.handle({
			move |input, event| match event {
				Event::KeyUp => {
					match input.value().parse::<f32>() {
						Ok(value) => {
							if input.value().len() > 3 {
								input.set_value(&value.to_string());
							}
							if value > 255.0 {
								input.set_value("255");
							}
							stop_signals.store_true();
							tx.send(Message::Refresh).unwrap();
						}
						Err(_) => {
							input.set_value("0");
						}
					}
					true
				}
				_ => true,
			}
		});

		color_input
	}
}

#[derive(Clone)]
pub struct ColorTile {
	pub exterior_tile: Tile,
	pub toggle_button: ToggleButton,
	pub red_input: IntInput,
	pub green_input: IntInput,
	pub blue_input: IntInput,
}

impl ColorTile {
	pub fn input_activate(&mut self) {
		self.red_input.activate();
		self.green_input.activate();
		self.blue_input.activate();
	}

	pub fn input_deactivate(&mut self) {
		self.red_input.deactivate();
		self.green_input.deactivate();
		self.blue_input.deactivate();
	}

	pub fn activate(&mut self) {
		self.toggle_button.activate();
		if !self.toggle_button.is_toggled() {
			self.input_activate();
		}
	}

	pub fn deactivate(&mut self) {
		self.toggle_button.deactivate();
		self.input_deactivate();
	}

	pub fn set_state(&mut self, rgb_values: [u8; 3], button_toggle_state: bool) {
		self.red_input.set_value(rgb_values[0].to_string().as_str());
		self.green_input.set_value(rgb_values[1].to_string().as_str());
		self.blue_input.set_value(rgb_values[2].to_string().as_str());

		self.toggle_button.toggle(button_toggle_state);
		if button_toggle_state {
			self.deactivate();
		}
	}
}

impl ColorTile {
	pub fn new(x: i32, y: i32, tx: &flume::Sender<Message>, stop_signals: &StopSignals, master_tile: bool) -> Self {
		let button_size = 40;
		let inputs_offset = 60;

		let exterior_tile = Tile::new(x, y, TILE_WIDTH, TILE_HEIGHT, "");

		let button_tile = Tile::new(0, 0, TILE_HEIGHT, TILE_HEIGHT, "");
		let toggle_button = ToggleButton::new(0, 0, button_size, button_size, "").center_of_parent();
		button_tile.end();

		let inputs_tile = Tile::new(0, 0, TILE_WIDTH - TILE_HEIGHT, TILE_HEIGHT, "");
		let green_input = ColorInput::create(0, 0, 70, 40, BaseColor::Green, tx.clone(), stop_signals.clone()).center_of_parent();
		let red_input = ColorInput::create(0, 0, 70, 40, BaseColor::Red, tx.clone(), stop_signals.clone()).left_of(&green_input, inputs_offset);
		let blue_input = ColorInput::create(0, 0, 70, 40, BaseColor::Blue, tx.clone(), stop_signals.clone()).right_of(&green_input, inputs_offset);
		inputs_tile.end();

		let mut row = Pack::new(x, y, TILE_WIDTH, TILE_HEIGHT, "");
		row.set_type(PackType::Horizontal);
		row.end();

		row.add(&button_tile);
		row.add(&inputs_tile);

		let mut color_tile = Self {
			exterior_tile,
			toggle_button,
			red_input,
			green_input,
			blue_input,
		};

		color_tile.exterior_tile.end();

		color_tile.exterior_tile.set_frame(FrameType::FlatBox);
		if master_tile {
			color_tile.exterior_tile.set_color(Color::from_u32(Colors::LightGray as u32));
		} else {
			color_tile.exterior_tile.set_color(Color::from_u32(Colors::Gray as u32));

			color_tile.toggle_button.handle({
				let mut color_tile = color_tile.clone();
				let tx = tx.clone();
				let stop_signal = stop_signals.clone();
				move |button, event| match event {
					Event::Released => {
						if button.is_toggled() {
							color_tile.input_deactivate();
						} else {
							color_tile.input_activate();
						}
						stop_signal.store_true();
						tx.send(Message::Refresh).unwrap();
						true
					}
					_ => false,
				}
			});
		}

		color_tile.toggle_button.set_frame(FrameType::OFlatFrame);
		color_tile.toggle_button.set_color(Color::from_u32(Colors::White as u32));

		color_tile
	}
}

#[derive(Clone)]
pub struct ColorTiles {
	master: ColorTile,
	zones: [ColorTile; 4],
}

impl ColorTiles {
	pub fn new(x: i32, y: i32, tx: &flume::Sender<Message>, stop_signals: &StopSignals) -> Self {
		let mut column = Pack::new(x, y, TILE_WIDTH, TILE_HEIGHT, "");
		column.set_type(PackType::Vertical);
		column.end();

		let master = ColorTile::new(0, 0, &tx.clone(), &stop_signals.clone(), true);
		let left = ColorTile::new(0, 0, &tx.clone(), &stop_signals.clone(), false);
		let center_left = ColorTile::new(0, 0, &tx.clone(), &stop_signals.clone(), false);
		let center_right = ColorTile::new(0, 0, &tx.clone(), &stop_signals.clone(), false);
		let right = ColorTile::new(0, 0, &tx.clone(), &stop_signals.clone(), false);

		column.add(&right.exterior_tile);
		column.add(&center_right.exterior_tile);
		column.add(&center_left.exterior_tile);
		column.add(&left.exterior_tile);
		column.add(&master.exterior_tile);

		let mut color_tiles = Self {
			master,
			zones: [left, center_left, center_right, right],
		};

		color_tiles.master.toggle_button.handle({
			let mut color_tiles = color_tiles.clone();
			let mut master_tile = color_tiles.master.clone();
			let tx = tx.clone();
			let stop_signals = stop_signals.clone();
			move |button, event| match event {
				Event::Released => {
					if button.is_toggled() {
						master_tile.input_deactivate();
						color_tiles.zones_deactivate();
					} else {
						master_tile.input_activate();
						color_tiles.zones_activate();
					}
					stop_signals.store_true();
					tx.send(Message::Refresh).unwrap();
					true
				}
				_ => false,
			}
		});

		fn add_master_input_handle(color_tiles: &mut ColorTiles, color: BaseColor, tx: flume::Sender<Message>, stop_signals: StopSignals) {
			let mut input = match color {
				BaseColor::Red => color_tiles.master.red_input.clone(),
				BaseColor::Green => color_tiles.master.green_input.clone(),
				BaseColor::Blue => color_tiles.master.blue_input.clone(),
			};

			input.handle({
				let mut color_tiles = color_tiles.clone();
				move |input, event| match event {
					Event::KeyUp => {
						if let Ok(value) = input.value().parse::<f32>() {
							if input.value().len() > 3 {
								input.set_value(&value.to_string());
							}
							if value > 255.0 {
								input.set_value("255");
							}
							color_tiles.set_zones_value(color, input.value().parse().unwrap());
							stop_signals.store_true();
							tx.send(Message::Refresh).unwrap();
						} else {
							input.set_value("0");
							color_tiles.set_zones_value(color, 0);
						}
						true
					}
					_ => false,
				}
			});
		}

		add_master_input_handle(&mut color_tiles, BaseColor::Red, tx.clone(), stop_signals.clone());
		add_master_input_handle(&mut color_tiles, BaseColor::Green, tx.clone(), stop_signals.clone());
		add_master_input_handle(&mut color_tiles, BaseColor::Blue, tx.clone(), stop_signals.clone());

		color_tiles
	}

	pub fn zones_activate(&mut self) {
		self.zones[0].activate();
		self.zones[1].activate();
		self.zones[2].activate();
		self.zones[3].activate();
	}

	pub fn zones_deactivate(&mut self) {
		self.zones[0].deactivate();
		self.zones[1].deactivate();
		self.zones[2].deactivate();
		self.zones[3].deactivate();
	}

	pub fn activate(&mut self) {
		if self.master.toggle_button.is_toggled() {
			self.master.toggle_button.activate();
			self.zones_deactivate();
		} else {
			self.master.activate();
			self.zones_activate();
		}
	}

	pub fn deactivate(&mut self) {
		self.master.deactivate();
		self.zones_deactivate();
	}

	pub fn get_values(&mut self) -> [u8; 12] {
		if self.master.toggle_button.is_toggled() {
			[0; 12]
		} else {
			self.get_zones_values()
		}
	}

	pub fn get_zones_values(&mut self) -> [u8; 12] {
		let mut values = [0; 12];
		if !self.zones[0].toggle_button.is_toggled() {
			values[0] = self.zones[0].red_input.value().parse::<u8>().unwrap_or(255);
			values[1] = self.zones[0].green_input.value().parse::<u8>().unwrap_or(255);
			values[2] = self.zones[0].blue_input.value().parse::<u8>().unwrap_or(255);
		};
		if !self.zones[1].toggle_button.is_toggled() {
			values[3] = self.zones[1].red_input.value().parse::<u8>().unwrap_or(255);
			values[4] = self.zones[1].green_input.value().parse::<u8>().unwrap_or(255);
			values[5] = self.zones[1].blue_input.value().parse::<u8>().unwrap_or(255);
		};
		if !self.zones[2].toggle_button.is_toggled() {
			values[6] = self.zones[2].red_input.value().parse::<u8>().unwrap_or(255);
			values[7] = self.zones[2].green_input.value().parse::<u8>().unwrap_or(255);
			values[8] = self.zones[2].blue_input.value().parse::<u8>().unwrap_or(255);
		};
		if !self.zones[3].toggle_button.is_toggled() {
			values[9] = self.zones[3].red_input.value().parse::<u8>().unwrap_or(255);
			values[10] = self.zones[3].green_input.value().parse::<u8>().unwrap_or(255);
			values[11] = self.zones[3].blue_input.value().parse::<u8>().unwrap_or(255);
		};
		values
	}

	pub fn set_zones_value(&mut self, color: BaseColor, value: u8) {
		match color {
			BaseColor::Red => {
				self.zones[0].red_input.set_value(value.to_string().as_str());
				self.zones[1].red_input.set_value(value.to_string().as_str());
				self.zones[2].red_input.set_value(value.to_string().as_str());
				self.zones[3].red_input.set_value(value.to_string().as_str());
			}
			BaseColor::Green => {
				self.zones[0].green_input.set_value(value.to_string().as_str());
				self.zones[1].green_input.set_value(value.to_string().as_str());
				self.zones[2].green_input.set_value(value.to_string().as_str());
				self.zones[3].green_input.set_value(value.to_string().as_str());
			}
			BaseColor::Blue => {
				self.zones[0].blue_input.set_value(value.to_string().as_str());
				self.zones[1].blue_input.set_value(value.to_string().as_str());
				self.zones[2].blue_input.set_value(value.to_string().as_str());
				self.zones[3].blue_input.set_value(value.to_string().as_str());
			}
		}
	}

	pub fn update(&mut self, effect: Effects) {
		match effect {
			Effects::Static | Effects::Breath | Effects::Swipe | Effects::Fade => {
				self.activate();
			}
			Effects::Smooth | Effects::Wave | Effects::Lightning | Effects::AmbientLight | Effects::SmoothWave | Effects::Disco | Effects::Christmas | Effects::Temperature => {
				self.deactivate();
			}
		}
	}

	pub fn set_state(&mut self, rgb_array: &[u8; 12], buttons_toggle_state: [bool; 5], effect: Effects) {
		for (i, (_val, zone)) in rgb_array.iter().step_by(3).zip(self.zones.iter_mut()).enumerate() {
			let rgb_values: [u8; 3] = [rgb_array[i], rgb_array[i + 1], rgb_array[i + 2]];
			zone.set_state(rgb_values, buttons_toggle_state[i]);
		}
		self.update(effect);
	}
}
