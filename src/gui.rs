// #![windows_subsystem = "windows"]
use fltk::{
	app,
	browser::HoldBrowser,
	button::ToggleButton,
	enums::{Color, Event, Font, FrameType},
	group::{Pack, Tile},
	input::IntInput,
	menu::Choice,
	prelude::*,
	window::Window,
};

use std::sync::Arc;
use std::{thread, time};

use parking_lot::Mutex;
use rand::Rng;

const WIDTH: i32 = 900;
const HEIGHT: i32 = 450;

const WHITE: u32 = 0xffffff;

const RED: u32 = 0xff0000;
const GREEN: u32 = 0x00ff00;
const BLUE: u32 = 0x0000ff;
const DARK_GRAY: u32 = 0x333333;
const GRAY: u32 = 0x444444;
const LIGHT_GRAY: u32 = 0x777777;
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

fn new_rgb_controller_tile(master_tile: bool) -> RgbControlTile {
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

	if master_tile {
		control_tile.exterior_tile.set_color(Color::from_u32(LIGHT_GRAY));
	} else {
		control_tile.exterior_tile.set_color(Color::from_u32(GRAY));
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

pub fn start_ui() {
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
	let mut control_tiles = ControlTiles { master, control_sections };

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

	let keyboard: crate::keyboard_utils::Keyboard = match crate::keyboard_utils::get_keyboard() {
		Ok(keyboard) => keyboard,
		Err(err) => panic!("{}", err),
	};
	let keyboard = Arc::from(Mutex::from(keyboard));
	let effect_loop_is_active = Arc::new(Mutex::new(false));
	let thread_ended_signal = Arc::new(Mutex::new(true));

	//Begin app logic
	add_control_tile_handle(&mut control_tiles.control_sections.left, keyboard.clone(), 0, effect_loop_is_active.clone());
	add_control_tile_handle(&mut control_tiles.control_sections.center_left, keyboard.clone(), 1, effect_loop_is_active.clone());
	add_control_tile_handle(&mut control_tiles.control_sections.center_right, keyboard.clone(), 2, effect_loop_is_active.clone());
	add_control_tile_handle(&mut control_tiles.control_sections.right, keyboard.clone(), 3, effect_loop_is_active.clone());
	add_master_control_tile_handle(&mut control_tiles.clone(), keyboard.clone(), effect_loop_is_active.clone());

	// Effect choice
	effect_browser.set_callback({
		let keyboard = keyboard.clone();
		let thread_ended_signal = Arc::clone(&thread_ended_signal);
		let speed_choice = speed_choice.clone();
		move |browser| match browser.value() {
			0 => {
				browser.select(1);
			}
			_ => match effects_list[(browser.value() - 1) as usize] {
				"Static" => {
					control_tiles.activate();
					*effect_loop_is_active.lock() = false;
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Static);
					force_update_colors(&control_tiles.control_sections, &keyboard);
				}
				"Breath" => {
					control_tiles.activate();
					*effect_loop_is_active.lock() = false;
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Breath);
					force_update_colors(&control_tiles.control_sections, &keyboard);
				}
				"Smooth" => {
					control_tiles.deactivate();
					*effect_loop_is_active.lock() = false;
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Smooth);
				}
				"LeftWave" => {
					control_tiles.deactivate();
					*effect_loop_is_active.lock() = false;
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::LeftWave);
				}
				"RightWave" => {
					control_tiles.deactivate();
					*effect_loop_is_active.lock() = false;

					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::RightWave);
				}
				"LeftPulse" => {
					//Preparations
					*effect_loop_is_active.lock() = false;
					wait_thread_end(&thread_ended_signal);
					control_tiles.master_only();
					*effect_loop_is_active.lock() = true;
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Static);

					//Create necessary clones to be passed into thread
					let should_loop = Arc::clone(&effect_loop_is_active);
					let keyboard = Arc::clone(&keyboard);
					let speed_choice = Arc::from(Mutex::from(speed_choice.clone()));
					let control_tiles = Arc::from(Mutex::from(control_tiles.clone()));
					let signal = Arc::clone(&thread_ended_signal);
					thread::spawn(move || {
						*signal.lock() = false;
						while *should_loop.lock() {
							let master_tile = &control_tiles.lock().master;
							let red = master_tile.red_input.value().parse::<f32>().unwrap();
							let green = master_tile.green_input.value().parse::<f32>().unwrap();
							let blue = master_tile.blue_input.value().parse::<f32>().unwrap();

							let color: [f32; 12] = [red, green, blue, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
							keyboard.lock().transition_colors_to(&color, 255 / speed_choice.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							if !*should_loop.lock() {
								break;
							}
							let color: [f32; 12] = [0.0, 0.0, 0.0, red, green, blue, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
							keyboard.lock().transition_colors_to(&color, 255 / speed_choice.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							if !*should_loop.lock() {
								break;
							}
							let color: [f32; 12] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, red, green, blue, 0.0, 0.0, 0.0];
							keyboard.lock().transition_colors_to(&color, 255 / speed_choice.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							if !*should_loop.lock() {
								break;
							}
							let color: [f32; 12] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, red, green, blue];
							keyboard.lock().transition_colors_to(&color, 255 / speed_choice.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							thread::sleep(time::Duration::from_millis(50));
						}
						*signal.lock() = true;
					});
				}
				"RightPulse" => {
					//Preparations
					*effect_loop_is_active.lock() = false;
					wait_thread_end(&thread_ended_signal);
					control_tiles.master_only();
					*effect_loop_is_active.lock() = true;
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Static);

					//Create necessary clones to be passed into thread
					let should_loop = Arc::clone(&effect_loop_is_active);
					let keyboard = Arc::clone(&keyboard);
					let speed_choice = Arc::from(Mutex::from(speed_choice.clone()));
					let control_tiles = Arc::from(Mutex::from(control_tiles.clone()));
					let signal = Arc::clone(&thread_ended_signal);
					thread::spawn(move || {
						*signal.lock() = false;
						while *should_loop.lock() {
							let master_tile = &control_tiles.lock().master;
							let red = master_tile.red_input.value().parse::<f32>().unwrap();
							let green = master_tile.green_input.value().parse::<f32>().unwrap();
							let blue = master_tile.blue_input.value().parse::<f32>().unwrap();
							let color: [f32; 12] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, red, green, blue];
							keyboard.lock().transition_colors_to(&color, 255 / speed_choice.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							if !*should_loop.lock() {
								break;
							}
							let color: [f32; 12] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, red, green, blue, 0.0, 0.0, 0.0];
							keyboard.lock().transition_colors_to(&color, 255 / speed_choice.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							if !*should_loop.lock() {
								break;
							}
							let color: [f32; 12] = [0.0, 0.0, 0.0, red, green, blue, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
							keyboard.lock().transition_colors_to(&color, 255 / speed_choice.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							if !*should_loop.lock() {
								break;
							}
							let color: [f32; 12] = [red, green, blue, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
							keyboard.lock().transition_colors_to(&color, 255 / speed_choice.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							thread::sleep(time::Duration::from_millis(50));
						}
						*signal.lock() = true;
					});
				}
				"Lightning" => {
					//Preparations
					*effect_loop_is_active.lock() = false;
					wait_thread_end(&thread_ended_signal);
					control_tiles.deactivate();
					*effect_loop_is_active.lock() = true;
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Static);

					//Create necessary clones to be passed into thread
					let should_loop = Arc::clone(&effect_loop_is_active);
					let keyboard = Arc::clone(&keyboard);
					let signal = Arc::clone(&thread_ended_signal);
					thread::spawn(move || {
						*signal.lock() = false;
						let zone = 0;
						let mut zone_repeat_count = 0;
						while *should_loop.lock() {
							let mut next_zone = rand::thread_rng().gen_range(0..4);
							if next_zone == zone {
								zone_repeat_count += 1;
								if zone_repeat_count > 1 {
									while next_zone == zone {
										next_zone = rand::thread_rng().gen_range(0..4);
									}
									zone_repeat_count = 0;
								}
							}
							let zone = next_zone;
							let steps = rand::thread_rng().gen_range(50..=200);
							keyboard.lock().set_zone_by_index(zone, [255.0; 3]);
							// keyboard.lock().set_colors_to(&[255.0; 12]);
							keyboard.lock().transition_colors_to(&[0.0; 12], steps, 5);
							let sleep_time = rand::thread_rng().gen_range(100..=2000);
							thread::sleep(time::Duration::from_millis(sleep_time));
						}
						*signal.lock() = true;
					});
				}
				_ => {}
			},
		}
	});

	//Speed
	speed_choice.set_callback({
		let keyboard = keyboard.clone();
		move |choice| match choice.choice() {
			Some(value) => {
				let speed = value.parse::<u8>().unwrap();
				if (1..=4).contains(&speed) {
					keyboard.lock().set_speed(speed);
				}
			}
			_ => {}
		}
	});

	//Brightness
	brightness_choice.set_callback({
		move |choice| match choice.choice() {
			Some(value) => {
				let brightness = value.parse::<u8>().unwrap();
				if (1..=2).contains(&brightness) {
					keyboard.lock().set_brightness(brightness);
				}
			}
			_ => {}
		}
	});

	app.run().unwrap();
}

fn add_control_tile_handle(control_tile: &mut RgbControlTile, keyboard: Arc<Mutex<crate::keyboard_utils::Keyboard>>, index: u8, effect_loop_is_active: Arc<Mutex<bool>>) {
	//Button
	control_tile.toggle_button.handle({
		let keyboard = keyboard.clone();
		let mut control_tile = control_tile.clone();
		move |button, event| match event {
			Event::Released => {
				match button.is_toggled() {
					true => {
						keyboard.lock().set_zone_by_index(index, [0.0; 3]);
						control_tile.red_input.deactivate();
						control_tile.green_input.deactivate();
						control_tile.blue_input.deactivate();
					}
					false => {
						keyboard.lock().set_zone_by_index(
							index,
							[
								control_tile.red_input.value().parse::<f32>().unwrap(),
								control_tile.green_input.value().parse::<f32>().unwrap(),
								control_tile.blue_input.value().parse::<f32>().unwrap(),
							],
						);
						control_tile.red_input.activate();
						control_tile.green_input.activate();
						control_tile.blue_input.activate();
					}
				}
				true
			}
			_ => false,
		}
	});

	fn add_input_handle(input: &mut IntInput, color: BaseColor, keyboard: Arc<Mutex<crate::keyboard_utils::Keyboard>>, triplet_index: u8, effect_loop_is_active: Arc<Mutex<bool>>) {
		let index = match color {
			BaseColor::Red => 0,
			BaseColor::Green => 1,
			BaseColor::Blue => 2,
		};
		input.handle({
			let keyboard = keyboard;
			let effect_loop_is_active = Arc::clone(&effect_loop_is_active);
			move |input, event| match event {
				Event::KeyUp => {
					match input.value().parse::<f32>() {
						Ok(val) => {
							input.set_value(&val.to_string());
							if val > 255.0 {
								input.set_value("255");
								if !*effect_loop_is_active.lock() {
									keyboard.lock().set_value_by_index(triplet_index, index, 255.0);
								}
							} else {
								if !*effect_loop_is_active.lock() {
									keyboard.lock().set_value_by_index(triplet_index, index, val);
								}
							}
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

	//Red
	add_input_handle(&mut control_tile.red_input, BaseColor::Red, keyboard.clone(), index, effect_loop_is_active.clone());
	//Green
	add_input_handle(&mut control_tile.green_input, BaseColor::Green, keyboard.clone(), index, effect_loop_is_active.clone());
	//Blue
	add_input_handle(&mut control_tile.blue_input, BaseColor::Blue, keyboard, index, effect_loop_is_active);
}

fn add_master_control_tile_handle(control_tiles: &mut ControlTiles, keyboard: Arc<Mutex<crate::keyboard_utils::Keyboard>>, effect_loop_is_active: Arc<Mutex<bool>>) {
	let mut master_tile = control_tiles.master.clone();

	//Button
	master_tile.toggle_button.handle({
		let keyboard = keyboard.clone();
		let mut control_tiles = control_tiles.clone();
		let mut master_tile = master_tile.clone();
		move |button, event| match event {
			Event::Released => {
				match button.is_toggled() {
					true => {
						keyboard.lock().set_colors_to(&[0.0; 12]);
						master_tile.red_input.deactivate();
						master_tile.green_input.deactivate();
						master_tile.blue_input.deactivate();
						control_tiles.control_sections.deactivate();
					}
					false => {
						force_update_colors(&control_tiles.control_sections, &keyboard);
						master_tile.red_input.activate();
						master_tile.green_input.activate();
						master_tile.blue_input.activate();
						control_tiles.control_sections.activate();
					}
				}
				true
			}
			_ => false,
		}
	});

	fn add_master_input_handle(input: &mut IntInput, color: BaseColor, keyboard: Arc<Mutex<crate::keyboard_utils::Keyboard>>, control_tiles: ControlTiles, effect_loop_is_active: Arc<Mutex<bool>>) {
		let index = match color {
			BaseColor::Red => 0,
			BaseColor::Green => 1,
			BaseColor::Blue => 2,
		};
		input.handle({
			let keyboard = keyboard;
			let mut control_tiles = control_tiles;
			let effect_loop_is_active = Arc::clone(&effect_loop_is_active);
			move |input, event| match event {
				Event::KeyUp => {
					match input.value().parse::<f32>() {
						Ok(val) => {
							input.set_value(&val.to_string());
							if val > 255.0 {
								input.set_value("255");
								if !*effect_loop_is_active.lock() {
									keyboard.lock().solid_set_value_by_index(index, 255.0);
								}
								control_tiles.control_sections.change_color_value(color, 255.0);
							} else {
								if !*effect_loop_is_active.lock() {
									keyboard.lock().solid_set_value_by_index(index, val);
								}
								control_tiles.control_sections.change_color_value(color, val);
							}
						}
						Err(_) => {
							input.set_value("0");
							control_tiles.control_sections.change_color_value(color, 0.0);
						}
					}
					true
				}
				_ => false,
			}
		});
	}

	//Red
	add_master_input_handle(&mut master_tile.red_input, BaseColor::Red, keyboard.clone(), control_tiles.clone(), effect_loop_is_active.clone());
	//Green
	add_master_input_handle(&mut master_tile.green_input, BaseColor::Green, keyboard.clone(), control_tiles.clone(), effect_loop_is_active.clone());
	//Blue
	add_master_input_handle(&mut master_tile.blue_input, BaseColor::Blue, keyboard, control_tiles.clone(), effect_loop_is_active);
}

fn force_update_colors(sections: &SectionControlTiles, keyboard: &Arc<Mutex<crate::keyboard_utils::Keyboard>>) {
	let target = [
		sections.left.red_input.value().parse::<f32>().unwrap(),
		sections.left.green_input.value().parse::<f32>().unwrap(),
		sections.left.blue_input.value().parse::<f32>().unwrap(),
		sections.center_left.red_input.value().parse::<f32>().unwrap(),
		sections.center_left.green_input.value().parse::<f32>().unwrap(),
		sections.center_left.blue_input.value().parse::<f32>().unwrap(),
		sections.center_right.red_input.value().parse::<f32>().unwrap(),
		sections.center_right.green_input.value().parse::<f32>().unwrap(),
		sections.center_right.blue_input.value().parse::<f32>().unwrap(),
		sections.right.red_input.value().parse::<f32>().unwrap(),
		sections.right.green_input.value().parse::<f32>().unwrap(),
		sections.right.blue_input.value().parse::<f32>().unwrap(),
	];
	keyboard.lock().set_colors_to(&target);
}

fn wait_thread_end(signal: &Arc<Mutex<bool>>) {
	while !*signal.lock() {
		thread::sleep(time::Duration::from_millis(100));
	}
}
