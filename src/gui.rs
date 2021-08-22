// #![windows_subsystem = "windows"]
use fltk::{
	app,
	browser::HoldBrowser,
	button::ToggleButton,
	enums::{Color, Font, FrameType},
	group::{Pack, Tile},
	input::IntInput,
	menu::Choice,
	prelude::*,
	window::Window,
};

const WIDTH: i32 = 900;
const HEIGHT: i32 = 450;

const WHITE: u32 = 0xffffff;

const RED: u32 = 0xff0000;
const GREEN: u32 = 0x00ff00;
const BLUE: u32 = 0x0000ff;
const DARK_GRAY: u32 = 0x333333;
const LIGHT_GRAY: u32 = 0x444444;
const LIGHTER_GRAY: u32 = 0xcccccc;

#[derive(Copy, Clone)]
pub enum BaseColor {
	Red,
	Green,
	Blue,
}
#[derive(Clone)]
pub struct AppUi {
	pub app: app::App,
	pub control_tiles: ControlTiles,
	pub effect_browser: HoldBrowser,
	pub speed: Choice,
	pub brightness: Choice,
	pub effects_list: Vec<&'static str>,
}
#[derive(Clone)]
pub struct ControlTiles {
	pub master: RgbControlTile,
	pub control_sections: SectionControlTiles,
}

impl ControlTiles {
	pub fn activate(&mut self) {
		self.master.activate();
		self.control_sections.activate();
	}
	pub fn deactivate(&mut self) {
		self.master.deactivate();
		self.control_sections.deactivate();
	}
	pub fn master_only(&mut self) {
		self.deactivate();
		self.master.activate();
		self.master.toggle_button.deactivate();
	}
}
#[derive(Clone)]
pub struct SectionControlTiles {
	pub left: RgbControlTile,
	pub center_left: RgbControlTile,
	pub center_right: RgbControlTile,
	pub right: RgbControlTile,
}

impl SectionControlTiles {
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
}

#[derive(Clone)]
pub struct RgbControlTile {
	pub exterior_tile: Tile,
	pub toggle_button: ToggleButton,
	pub red_input: IntInput,
	pub green_input: IntInput,
	pub blue_input: IntInput,
}

impl RgbControlTile {
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

fn new_rgb_controller_tile(masterTile: bool) -> RgbControlTile {
	let center_x = 540 / 2;
	let center_y = 90 / 2 - 20;
	let offset = 120;

	//Begin tile
	let mut control_tile = RgbControlTile {
		exterior_tile: Tile::new(0, 0, 540, 90, ""),
		toggle_button: ToggleButton::new(25, 25, 40, 40, ""),
		red_input: IntInput::new(center_x - offset, center_y, 60, 40, "R:"),
		green_input: IntInput::new(center_x, center_y, 60, 40, "G:"),
		blue_input: IntInput::new(center_x + offset, center_y, 60, 40, "B:"),
	};

	control_tile.exterior_tile.add(&control_tile.toggle_button);

	control_tile.exterior_tile.add(&control_tile.red_input);
	control_tile.exterior_tile.add(&control_tile.green_input);
	control_tile.exterior_tile.add(&control_tile.blue_input);
	control_tile.exterior_tile.end();
	//Themeing
	control_tile.exterior_tile.set_frame(FrameType::FlatBox);

	if masterTile {
		control_tile.exterior_tile.set_color(Color::from_u32(0x777777));
	} else {
		control_tile.exterior_tile.set_color(Color::from_u32(LIGHT_GRAY));
	}

	//Button
	control_tile.toggle_button.set_frame(FrameType::OFlatFrame);
	control_tile.toggle_button.set_color(Color::from_u32(WHITE));

	//Inputs
	fn theme_input(input: &mut IntInput, color: BaseColor) {
		match color {
			BaseColor::Red => input.set_label_color(Color::from_u32(RED)),
			BaseColor::Green => input.set_label_color(Color::from_u32(GREEN)),
			BaseColor::Blue => input.set_label_color(Color::from_u32(BLUE)),
		}
		input.set_frame(FrameType::FlatBox);
		input.set_color(Color::from_u32(DARK_GRAY));
		input.set_selection_color(Color::White);
		input.set_text_color(Color::from_u32(WHITE));
		input.set_text_size(30);
		input.set_label_size(30);
		input.set_value("0");
	}

	//Red
	theme_input(&mut control_tile.red_input, BaseColor::Red);

	//Green
	theme_input(&mut control_tile.green_input, BaseColor::Green);

	//Blue
	theme_input(&mut control_tile.blue_input, BaseColor::Blue);

	control_tile
}

pub fn create_ui() -> AppUi {
	let app = app::App::default();
	let mut win = Window::default().with_size(WIDTH, HEIGHT).with_label("Legion 5 Pro Light Control Thing");
	let mut color_picker_pack = Pack::new(0, 0, 540, 360, "");
	let master = new_rgb_controller_tile(true);
	let control_sections: SectionControlTiles = SectionControlTiles {
		left: (new_rgb_controller_tile(false)),
		center_left: (new_rgb_controller_tile(false)),
		center_right: (new_rgb_controller_tile(false)),
		right: (new_rgb_controller_tile(false)),
	};
	let control_tiles = ControlTiles { master, control_sections };

	color_picker_pack.add(&control_tiles.control_sections.left.exterior_tile);
	color_picker_pack.add(&control_tiles.control_sections.center_left.exterior_tile);
	color_picker_pack.add(&control_tiles.control_sections.center_right.exterior_tile);
	color_picker_pack.add(&control_tiles.control_sections.right.exterior_tile);
	color_picker_pack.add(&control_tiles.master.exterior_tile);
	color_picker_pack.end();

	let mut effect_type_tile = Tile::new(540, 0, 360, 360, "");
	let mut effect_browser = HoldBrowser::new(0, 0, 310, 310, "").center_of_parent();
	let effects_list: Vec<&str> = vec!["Static", "Breath", "Smooth", "LeftWave", "RightWave", "LeftPulse", "RightPulse", "Lightning"];
	for effect in effects_list.iter() {
		effect_browser.add(effect);
	}
	effect_type_tile.end();
	let mut options_tile = Tile::new(540, 360, 360, 90, "");
	let mut speed_choice = Choice::new(540 + 100, 385, 40, 40, "Speed:");
	speed_choice.add_choice("1|2|3|4");
	let mut brightness_choice = Choice::new(0, 0, 40, 40, "Brightness:").right_of(&speed_choice, 140);
	brightness_choice.add_choice("1|2");
	options_tile.end();
	win.end();
	win.make_resizable(false);
	win.show();

	// Theming
	app::background(51, 51, 51);
	app::set_visible_focus(false);
	app::set_font(Font::HelveticaBold);

	effect_type_tile.set_frame(FrameType::FlatBox);
	effect_type_tile.set_color(Color::from_u32(0x222222));

	// Effect choice
	effect_browser.set_frame(FrameType::FlatBox);
	effect_browser.set_color(Color::from_u32(LIGHTER_GRAY));
	effect_browser.set_selection_color(Color::from_u32(WHITE));
	effect_browser.set_text_size(20);
	effect_browser.select(1);

	// Options tile
	options_tile.set_frame(FrameType::FlatBox);
	// options_tile.set_color(Color::from_u32(0xf00000));

	//Speed choice
	speed_choice.set_frame(FrameType::FlatBox);
	speed_choice.set_color(Color::from_u32(DARK_GRAY));
	speed_choice.set_label_color(Color::from_u32(WHITE));
	speed_choice.set_selection_color(Color::White);
	speed_choice.set_text_color(Color::from_u32(WHITE));
	speed_choice.set_text_size(20);
	speed_choice.set_label_size(20);
	speed_choice.set_value(0);

	//Brightness choice
	brightness_choice.set_frame(FrameType::FlatBox);
	brightness_choice.set_color(Color::from_u32(DARK_GRAY));
	brightness_choice.set_label_color(Color::from_u32(WHITE));
	brightness_choice.set_selection_color(Color::White);
	brightness_choice.set_text_color(Color::from_u32(WHITE));
	brightness_choice.set_text_size(20);
	brightness_choice.set_label_size(20);
	brightness_choice.set_value(0);

	AppUi {
		app,
		control_tiles,
		effect_browser,
		speed: speed_choice,
		brightness: brightness_choice,
		effects_list,
	}
}
