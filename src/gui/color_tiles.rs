use super::enums::{BaseColor, Colors};
use crate::{
	enums::{Effects, Message},
	gui::color_tiles,
	keyboard_manager::StopSignals,
};
use fltk::{
	button::ToggleButton,
	enums::{Color, Event, FrameType},
	group::Tile,
	input::IntInput,
	prelude::*,
};
use serde::{Deserialize, Serialize};
use std::sync::mpsc;

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
		color_input.set_frame(FrameType::RFlatBox);
		color_input.set_color(Color::from_u32(Colors::DarkGray as u32));
		color_input.set_selection_color(Color::White);
		color_input.set_text_color(Color::from_u32(Colors::White as u32));
		color_input.set_text_size(30);
		color_input.set_label_size(30);
		color_input.set_value("0");
		color_input.set_maximum_size(4);
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
	pub fn activate(&mut self) {}
	pub fn deactivate(&mut self) {}

	pub fn set_state(&mut self) {}
	pub fn get_state(&mut self) {}
}

impl ColorTile {
	pub fn create(x: i32, y: i32, master_tile: bool) -> Self {
		let button_size = 40;
		let inputs_offset = 60;

		let exterior_tile = Tile::new(x, y, TILE_WIDTH, TILE_HEIGHT, "");
		let toggle_button = ToggleButton::new(x + 25, y + TILE_HEIGHT / 2 - button_size / 2, button_size, button_size, "");
		let inputs_tile = Tile::new(x + TILE_HEIGHT, y, TILE_WIDTH - TILE_HEIGHT, TILE_HEIGHT, "");
		let green_input = ColorInput::create(0, 0, 70, 40, BaseColor::Green).center_of_parent();
		let red_input = ColorInput::create(0, 0, 70, 40, BaseColor::Red).left_of(&green_input, inputs_offset);
		let blue_input = ColorInput::create(0, 0, 70, 40, BaseColor::Blue).right_of(&green_input, inputs_offset);
		inputs_tile.end();

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
		}

		color_tile.toggle_button.set_frame(FrameType::OFlatFrame);
		color_tile.toggle_button.set_color(Color::from_u32(Colors::White as u32));
		color_tile
	}
	pub fn get_values(&mut self) {}
}

#[derive(Clone)]
pub struct Zones {
	pub left: ColorTile,
	pub center_left: ColorTile,
	pub center_right: ColorTile,
	pub right: ColorTile,
}

impl Zones {
	pub fn create(x: i32, y: i32) -> Self {
		Zones {
			left: ColorTile::create(x, y, false),
			center_left: ColorTile::create(x, y + TILE_HEIGHT, false),
			center_right: ColorTile::create(x, y + TILE_HEIGHT * 2, false),
			right: ColorTile::create(x, y + TILE_HEIGHT * 3, false),
		}
	}
	pub fn activate(&mut self) {}
	pub fn deactivate(&mut self) {}
	pub fn change_color_value(&mut self, color: BaseColor, value: u8) {}
	pub fn get_values(&mut self) {}
}

#[derive(Clone)]
pub struct ColorTiles {
	master: ColorTile,
	zones: Zones,
}

impl ColorTiles {
	pub fn new(x: i32, y: i32, tx: &mpsc::Sender<Message>, stop_signals: StopSignals) -> Self {
		let color_tiles = Self {
			master: (color_tiles::ColorTile::create(x, y + TILE_HEIGHT * 4, true)),
			zones: color_tiles::Zones::create(x, y),
		};

		color_tiles
	}

	pub fn activate(&mut self) {}

	pub fn deactivate(&mut self) {}

	pub fn master_only(&mut self) {}

	pub fn set_state(&mut self) {}

	pub fn get_state(&mut self) {}

	pub fn get_zone_values(&mut self) {}

	pub fn update(&mut self, effect: Effects) {}
}
