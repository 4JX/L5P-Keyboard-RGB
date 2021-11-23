use std::sync::{
	atomic::{AtomicBool, Ordering},
	mpsc, Arc,
};

use crate::{
	enums::{Effects, Message},
	gui::color_tiles,
};

use super::enums::{BaseColor, Colors};
use fltk::{
	button::ToggleButton,
	enums::{Color, Event, FrameType},
	group::Tile,
	input::IntInput,
	prelude::*,
};
use serde::{Deserialize, Serialize};

const TILE_WIDTH: i32 = 540;
const TILE_HEIGHT: i32 = 90;

pub struct ColorInput;

impl ColorInput {
	pub fn create(x: i32, y: i32, width: i32, height: i32, color: BaseColor) -> IntInput {
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
		color_input.set_frame(FrameType::FlatBox);
		color_input.set_color(Color::from_u32(Colors::DarkGray as u32));
		color_input.set_selection_color(Color::White);
		color_input.set_text_color(Color::from_u32(Colors::White as u32));
		color_input.set_text_size(30);
		color_input.set_label_size(30);
		color_input.set_value("0");
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

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct ColorTileState {
	pub rgb_values: [u8; 3],
	pub button_toggle_state: bool,
}

impl ColorTile {
	pub fn activate(&mut self) {
		self.toggle_button.activate();
		if !self.toggle_button.is_toggled() {
			self.red_input.activate();
			self.green_input.activate();
			self.blue_input.activate();
		}
	}
	pub fn deactivate(&mut self) {
		self.toggle_button.deactivate();
		self.red_input.deactivate();
		self.green_input.deactivate();
		self.blue_input.deactivate();
	}

	pub fn set_state(&mut self, state: ColorTileState) {
		self.red_input.set_value(state.rgb_values[0].to_string().as_str());
		self.green_input.set_value(state.rgb_values[1].to_string().as_str());
		self.blue_input.set_value(state.rgb_values[2].to_string().as_str());

		self.toggle_button.toggle(state.button_toggle_state);
		if state.button_toggle_state {
			self.deactivate();
		}
	}
	pub fn get_state(&mut self) -> ColorTileState {
		ColorTileState {
			rgb_values: self.get_values(),
			button_toggle_state: self.toggle_button.is_toggled(),
		}
	}
}

#[allow(dead_code)]
impl ColorTile {
	pub fn create(x: i32, y: i32, master_tile: bool) -> Self {
		let button_size = 40;
		let inputs_offset = 70;

		//Begin tile
		let exterior_tile = Tile::new(x, y, TILE_WIDTH, TILE_HEIGHT, "");
		let toggle_button = ToggleButton::new(x + 25, y + TILE_HEIGHT / 2 - button_size / 2, button_size, button_size, "");
		let inputs_tile = Tile::new(x + TILE_HEIGHT, y, TILE_WIDTH - TILE_HEIGHT, TILE_HEIGHT, "");
		let green_input = ColorInput::create(0, 0, 60, 40, BaseColor::Green).center_of_parent();
		let red_input = ColorInput::create(0, 0, 60, 40, BaseColor::Red).left_of(&green_input, inputs_offset);
		let blue_input = ColorInput::create(0, 0, 60, 40, BaseColor::Blue).right_of(&green_input, inputs_offset);
		inputs_tile.end();

		let mut color_tile = Self {
			exterior_tile,
			toggle_button,
			red_input,
			green_input,
			blue_input,
		};

		color_tile.exterior_tile.end();

		//Theming
		color_tile.exterior_tile.set_frame(FrameType::FlatBox);
		if master_tile {
			color_tile.exterior_tile.set_color(Color::from_u32(Colors::LightGray as u32));
		} else {
			color_tile.exterior_tile.set_color(Color::from_u32(Colors::Gray as u32));
		}

		//Button
		color_tile.toggle_button.set_frame(FrameType::OFlatFrame);
		color_tile.toggle_button.set_color(Color::from_u32(Colors::White as u32));
		color_tile
	}
	pub fn get_values(&mut self) -> [u8; 3] {
		let mut values = [0; 3];
		if !self.toggle_button.is_toggled() {
			values[0] = self.red_input.value().parse::<u8>().unwrap_or(255);
			values[1] = self.green_input.value().parse::<u8>().unwrap_or(255);
			values[2] = self.blue_input.value().parse::<u8>().unwrap_or(255);
		};
		values
	}
}

#[derive(Clone)]
pub struct Zones {
	pub left: ColorTile,
	pub center_left: ColorTile,
	pub center_right: ColorTile,
	pub right: ColorTile,
}

#[allow(dead_code)]
impl Zones {
	pub fn create(x: i32, y: i32) -> Self {
		Zones {
			left: ColorTile::create(x, y, false),
			center_left: ColorTile::create(x, y + TILE_HEIGHT, false),
			center_right: ColorTile::create(x, y + TILE_HEIGHT * 2, false),
			right: ColorTile::create(x, y + TILE_HEIGHT * 3, false),
		}
	}
	pub fn activate(&mut self) {
		self.left.activate();
		self.center_left.activate();
		self.center_right.activate();
		self.right.activate();
	}
	pub fn deactivate(&mut self) {
		self.left.deactivate();
		self.center_left.deactivate();
		self.center_right.deactivate();
		self.right.deactivate();
	}
	pub fn change_color_value(&mut self, color: BaseColor, value: u8) {
		match color {
			BaseColor::Red => {
				self.left.red_input.set_value(value.to_string().as_str());
				self.center_left.red_input.set_value(value.to_string().as_str());
				self.center_right.red_input.set_value(value.to_string().as_str());
				self.right.red_input.set_value(value.to_string().as_str());
			}
			BaseColor::Green => {
				self.left.green_input.set_value(value.to_string().as_str());
				self.center_left.green_input.set_value(value.to_string().as_str());
				self.center_right.green_input.set_value(value.to_string().as_str());
				self.right.green_input.set_value(value.to_string().as_str());
			}
			BaseColor::Blue => {
				self.left.blue_input.set_value(value.to_string().as_str());
				self.center_left.blue_input.set_value(value.to_string().as_str());
				self.center_right.blue_input.set_value(value.to_string().as_str());
				self.right.blue_input.set_value(value.to_string().as_str());
			}
		}
	}
	pub fn get_values(&mut self) -> [u8; 12] {
		let mut values = [0; 12];
		if !self.left.toggle_button.is_toggled() {
			values[0] = self.left.red_input.value().parse::<u8>().unwrap_or(255);
			values[1] = self.left.green_input.value().parse::<u8>().unwrap_or(255);
			values[2] = self.left.blue_input.value().parse::<u8>().unwrap_or(255);
		};
		if !self.center_left.toggle_button.is_toggled() {
			values[3] = self.center_left.red_input.value().parse::<u8>().unwrap_or(255);
			values[4] = self.center_left.green_input.value().parse::<u8>().unwrap_or(255);
			values[5] = self.center_left.blue_input.value().parse::<u8>().unwrap_or(255);
		};
		if !self.center_right.toggle_button.is_toggled() {
			values[6] = self.center_right.red_input.value().parse::<u8>().unwrap_or(255);
			values[7] = self.center_right.green_input.value().parse::<u8>().unwrap_or(255);
			values[8] = self.center_right.blue_input.value().parse::<u8>().unwrap_or(255);
		};
		if !self.right.toggle_button.is_toggled() {
			values[9] = self.right.red_input.value().parse::<u8>().unwrap_or(255);
			values[10] = self.right.green_input.value().parse::<u8>().unwrap_or(255);
			values[11] = self.right.blue_input.value().parse::<u8>().unwrap_or(255);
		};
		values
	}
}

#[derive(Clone)]
pub struct ColorTiles {
	pub master: ColorTile,
	pub zones: Zones,
}

#[derive(Serialize, Deserialize)]
pub struct ColorTilesState {
	pub master: ColorTileState,
	pub left: ColorTileState,
	pub center_left: ColorTileState,
	pub center_right: ColorTileState,
	pub right: ColorTileState,
}

#[allow(dead_code)]
impl ColorTiles {
	pub fn new(x: i32, y: i32, tx: &mpsc::Sender<Message>, stop_signal: Arc<AtomicBool>) -> Self {
		fn add_zone_tile_handle(color_tile: &mut color_tiles::ColorTile, tx: &mpsc::Sender<Message>, stop_signal: Arc<AtomicBool>) {
			fn add_input_handle(input: &mut IntInput, tx: mpsc::Sender<Message>, stop_signal: Arc<AtomicBool>) {
				input.handle({
					move |input, event| match event {
						Event::KeyUp => {
							match input.value().parse::<f32>() {
								Ok(value) => {
									input.set_value(&value.to_string());
									if value > 255.0 {
										input.set_value("255");
									}
									stop_signal.store(true, Ordering::SeqCst);
									tx.send(Message::Refresh).unwrap();
								}
								Err(_) => {
									input.set_value("0");
								}
							}
							true
						}
						_ => false,
					}
				});
			}

			color_tile.toggle_button.handle({
				let mut color_tile = color_tile.clone();
				let tx = tx.clone();
				let stop_signal = stop_signal.clone();
				move |button, event| match event {
					Event::Released => {
						if button.is_toggled() {
							color_tile.red_input.deactivate();
							color_tile.green_input.deactivate();
							color_tile.blue_input.deactivate();
						} else {
							color_tile.red_input.activate();
							color_tile.green_input.activate();
							color_tile.blue_input.activate();
						}
						stop_signal.store(true, Ordering::SeqCst);
						tx.send(Message::Refresh).unwrap();
						true
					}
					_ => false,
				}
			});

			add_input_handle(&mut color_tile.red_input, tx.clone(), stop_signal.clone());
			add_input_handle(&mut color_tile.green_input, tx.clone(), stop_signal.clone());
			add_input_handle(&mut color_tile.blue_input, tx.clone(), stop_signal);
		}

		let mut color_tiles = Self {
			master: (color_tiles::ColorTile::create(x, y + TILE_HEIGHT * 4, true)),
			zones: color_tiles::Zones::create(x, y),
		};

		add_zone_tile_handle(&mut color_tiles.zones.left, tx, stop_signal.clone());
		add_zone_tile_handle(&mut color_tiles.zones.center_left, tx, stop_signal.clone());
		add_zone_tile_handle(&mut color_tiles.zones.center_right, tx, stop_signal.clone());
		add_zone_tile_handle(&mut color_tiles.zones.right, tx, stop_signal.clone());

		fn add_master_input_handle(input: &mut IntInput, color: BaseColor, tx: mpsc::Sender<Message>, color_tiles: color_tiles::ColorTiles, stop_signal: Arc<AtomicBool>) {
			input.handle({
				let mut keyboard_color_tiles = color_tiles;
				move |input, event| match event {
					Event::KeyUp => {
						if let Ok(value) = input.value().parse::<f32>() {
							input.set_value(&value.to_string());
							if value > 255.0 {
								input.set_value("255");
							}
							keyboard_color_tiles.zones.change_color_value(color, input.value().parse().unwrap());
							stop_signal.store(true, Ordering::SeqCst);
							tx.send(Message::Refresh).unwrap();
						} else {
							input.set_value("0");
							keyboard_color_tiles.zones.change_color_value(color, 0);
						}
						true
					}
					_ => false,
				}
			});
		}
		let mut master_tile = color_tiles.master.clone();

		master_tile.toggle_button.handle({
			let mut keyboard_color_tiles = color_tiles.clone();
			let mut master_tile = master_tile.clone();
			let tx = tx.clone();
			let stop_signal = stop_signal.clone();
			move |button, event| match event {
				Event::Released => {
					if button.is_toggled() {
						master_tile.red_input.deactivate();
						master_tile.green_input.deactivate();
						master_tile.blue_input.deactivate();
						keyboard_color_tiles.zones.deactivate();
					} else {
						master_tile.red_input.activate();
						master_tile.green_input.activate();
						master_tile.blue_input.activate();
						keyboard_color_tiles.zones.activate();
					}
					stop_signal.store(true, Ordering::SeqCst);
					tx.send(Message::Refresh).unwrap();
					true
				}
				_ => false,
			}
		});

		add_master_input_handle(&mut master_tile.red_input, BaseColor::Red, tx.clone(), color_tiles.clone(), stop_signal.clone());
		add_master_input_handle(&mut master_tile.green_input, BaseColor::Green, tx.clone(), color_tiles.clone(), stop_signal.clone());
		add_master_input_handle(&mut master_tile.blue_input, BaseColor::Blue, tx.clone(), color_tiles.clone(), stop_signal);

		color_tiles
	}
	pub fn activate(&mut self) {
		if self.master.toggle_button.is_toggled() {
			self.master.toggle_button.activate();
			self.zones.deactivate();
		} else {
			self.master.activate();
			self.zones.activate();
		}
	}
	pub fn deactivate(&mut self) {
		self.master.deactivate();
		self.zones.deactivate();
	}
	pub fn master_only(&mut self) {
		self.deactivate();
		self.master.activate();
		self.master.toggle_button.deactivate();
	}
	pub fn set_state(&mut self, state: &ColorTilesState, effect: Effects) {
		self.master.set_state(state.master);
		self.zones.left.set_state(state.left);
		self.zones.center_left.set_state(state.center_left);
		self.zones.center_right.set_state(state.center_right);
		self.zones.right.set_state(state.right);
		self.update(effect);
	}

	pub fn get_state(&mut self) -> ColorTilesState {
		ColorTilesState {
			master: self.master.get_state(),
			left: self.zones.left.get_state(),
			center_left: self.zones.center_left.get_state(),
			center_right: self.zones.center_right.get_state(),
			right: self.zones.right.get_state(),
		}
	}

	pub fn get_zone_values(&mut self) -> [u8; 12] {
		let mut values = [0; 12];
		if !self.master.toggle_button.is_toggled() {
			values = self.zones.get_values();
		}
		values
	}

	pub fn update(&mut self, effect: Effects) {
		match effect {
			Effects::Static | Effects::Breath | Effects::LeftSwipe | Effects::RightSwipe => {
				self.activate();
			}
			Effects::Smooth
			| Effects::LeftWave
			| Effects::RightWave
			| Effects::Lightning
			| Effects::AmbientLight
			| Effects::SmoothLeftWave
			| Effects::SmoothRightWave
			| Effects::Disco
			| Effects::Christmas => {
				self.deactivate();
			}
		}
	}
}
