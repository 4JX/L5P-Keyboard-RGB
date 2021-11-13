use super::enums::{BaseColor, Colors};
use fltk::{
	button::ToggleButton,
	enums::{Color, FrameType},
	group::Tile,
	input::IntInput,
	prelude::*,
};

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

impl ColorTile {
	pub fn activate(&mut self) {
		self.toggle_button.activate();
		self.red_input.activate();
		self.green_input.activate();
		self.blue_input.activate();
	}
	pub fn deactivate(&mut self) {
		self.toggle_button.deactivate();
		self.red_input.deactivate();
		self.green_input.deactivate();
		self.blue_input.deactivate();
	}
}

impl ColorTile {
	pub fn create(master_tile: bool) -> Self {
		let center_x = 540 / 2;
		let center_y = 90 / 2 - 20;
		let offset = 120;
		//Begin tile
		let mut color_tile = Self {
			exterior_tile: Tile::new(0, 0, 540, 90, ""),
			toggle_button: ToggleButton::new(25, 25, 40, 40, ""),
			red_input: ColorInput::create(center_x - offset, center_y, 60, 40, BaseColor::Red),
			green_input: ColorInput::create(center_x, center_y, 60, 40, BaseColor::Green),
			blue_input: ColorInput::create(center_x + offset, center_y, 60, 40, BaseColor::Blue),
		};

		color_tile.exterior_tile.add(&color_tile.toggle_button);
		color_tile.exterior_tile.add(&color_tile.red_input);
		color_tile.exterior_tile.add(&color_tile.green_input);
		color_tile.exterior_tile.add(&color_tile.blue_input);
		color_tile.exterior_tile.end();

		//Themeing
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
	pub fn get_values(&mut self) -> [f32; 3] {
		let mut values = [0.0; 3];
		if !self.toggle_button.is_toggled() {
			values[0] = self.red_input.value().parse::<f32>().unwrap_or(0.0);
			values[1] = self.green_input.value().parse::<f32>().unwrap_or(0.0);
			values[2] = self.blue_input.value().parse::<f32>().unwrap_or(0.0);
		};
		values
	}
}

#[derive(Clone)]
pub struct ZoneColorTiles {
	pub left: ColorTile,
	pub center_left: ColorTile,
	pub center_right: ColorTile,
	pub right: ColorTile,
}

impl ZoneColorTiles {
	pub fn create() -> Self {
		ZoneColorTiles {
			left: ColorTile::create(false),
			center_left: ColorTile::create(false),
			center_right: ColorTile::create(false),
			right: ColorTile::create(false),
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
	pub fn change_color_value(&mut self, color: BaseColor, value: f32) {
		if !(0.0..=255.0).contains(&value) {
			panic!("Keyboard colors has value outside accepted range (0-255)");
		}
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
	pub fn get_values(&mut self) -> [f32; 12] {
		let mut values = [0.0; 12];
		if !self.left.toggle_button.is_toggled() {
			values[0] = self.left.red_input.value().parse::<f32>().unwrap_or(0.0);
			values[1] = self.left.green_input.value().parse::<f32>().unwrap_or(0.0);
			values[2] = self.left.blue_input.value().parse::<f32>().unwrap_or(0.0);
		};
		if !self.center_left.toggle_button.is_toggled() {
			values[3] = self.center_left.red_input.value().parse::<f32>().unwrap_or(0.0);
			values[4] = self.center_left.green_input.value().parse::<f32>().unwrap_or(0.0);
			values[5] = self.center_left.blue_input.value().parse::<f32>().unwrap_or(0.0);
		};
		if !self.center_right.toggle_button.is_toggled() {
			values[6] = self.center_right.red_input.value().parse::<f32>().unwrap_or(0.0);
			values[7] = self.center_right.green_input.value().parse::<f32>().unwrap_or(0.0);
			values[8] = self.center_right.blue_input.value().parse::<f32>().unwrap_or(0.0);
		};
		if !self.right.toggle_button.is_toggled() {
			values[9] = self.right.red_input.value().parse::<f32>().unwrap_or(0.0);
			values[10] = self.right.green_input.value().parse::<f32>().unwrap_or(0.0);
			values[11] = self.right.blue_input.value().parse::<f32>().unwrap_or(0.0);
		};
		values
	}
}

#[derive(Clone)]
pub struct KeyboardColorTiles {
	pub master: ColorTile,
	pub zones: ZoneColorTiles,
}

#[allow(dead_code)]
impl KeyboardColorTiles {
	pub fn activate(&mut self) {
		self.master.activate();
		self.zones.activate();
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
}
