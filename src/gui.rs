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

use image::buffer::ConvertBuffer;

use parking_lot::Mutex;

use rand::Rng;

use scrap::{Capturer, Display};

use std::convert::TryInto;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{
	thread,
	time::{Duration, Instant},
};

const DISP_WIDTH: u32 = 2560;
const DISP_HEIGHT: u32 = 1600;

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
pub struct ControlTiles {
	pub master: ZoneControlTile,
	pub zones: KeyboardZoneTiles,
}

#[allow(dead_code)]
impl ControlTiles {
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

#[derive(Clone)]
pub struct KeyboardZoneTiles {
	pub left: ZoneControlTile,
	pub center_left: ZoneControlTile,
	pub center_right: ZoneControlTile,
	pub right: ZoneControlTile,
}

impl KeyboardZoneTiles {
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
pub struct ZoneControlTile {
	pub exterior_tile: Tile,
	pub toggle_button: ToggleButton,
	pub red_input: IntInput,
	pub green_input: IntInput,
	pub blue_input: IntInput,
}

impl ZoneControlTile {
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

pub fn start_ui(keyboard: Arc<Mutex<crate::keyboard_utils::Keyboard>>) -> fltk::window::Window {
	//Keyboard
	let stop_signal = Arc::new(AtomicBool::new(true));
	let thread_ended_signal = Arc::new(AtomicBool::new(true));

	//UI
	let mut win = Window::default().with_size(WIDTH, HEIGHT).with_label("Legion 5 Pro Keyboard RGB Control");
	let mut color_picker_pack = Pack::new(0, 0, 540, 360, "");
	let mut control_tiles = create_control_tiles(keyboard.clone(), stop_signal.clone());

	color_picker_pack.add(&control_tiles.zones.left.exterior_tile);
	color_picker_pack.add(&control_tiles.zones.center_left.exterior_tile);
	color_picker_pack.add(&control_tiles.zones.center_right.exterior_tile);
	color_picker_pack.add(&control_tiles.zones.right.exterior_tile);
	color_picker_pack.add(&control_tiles.master.exterior_tile);
	color_picker_pack.end();

	let mut effect_type_tile = Tile::new(540, 0, 360, 360, "");
	let mut effect_browser = HoldBrowser::new(0, 0, 310, 310, "").center_of_parent();
	let effects_list: Vec<&str> = vec![
		"Static",
		"Breath",
		"Smooth",
		"LeftWave",
		"RightWave",
		"Lightning",
		"AmbientLight",
		"SmoothLeftWave",
		"SmoothRightWave",
		"LeftSwipe",
		"RightSwipe",
	];
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

	//Begin app logic
	// Effect choice
	effect_browser.set_callback({
		let keyboard = keyboard.clone();
		let thread_ended_signal = Arc::clone(&thread_ended_signal);
		let speed_choice = speed_choice.clone();
		move |browser| match browser.value() {
			0 => {
				browser.select(0);
			}
			_ => match effects_list[(browser.value() - 1) as usize] {
				"Static" => {
					control_tiles.activate();
					stop_signal.store(true, Ordering::Relaxed);
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Static);
					force_update_colors(&control_tiles.zones, &keyboard);
				}
				"Breath" => {
					control_tiles.activate();
					stop_signal.store(true, Ordering::Relaxed);
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Breath);
					force_update_colors(&control_tiles.zones, &keyboard);
				}
				"Smooth" => {
					control_tiles.deactivate();
					stop_signal.store(true, Ordering::Relaxed);
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Smooth);
				}
				"LeftWave" => {
					control_tiles.deactivate();
					stop_signal.store(true, Ordering::Relaxed);
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::LeftWave);
				}
				"RightWave" => {
					control_tiles.deactivate();
					stop_signal.store(true, Ordering::Relaxed);

					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::RightWave);
				}
				"Lightning" => {
					//Preparations
					stop_signal.store(true, Ordering::Relaxed);
					wait_thread_end(&thread_ended_signal);
					control_tiles.deactivate();
					stop_signal.store(false, Ordering::Relaxed);
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Static);

					//Create necessary clones to be passed into thread
					let stop_signal = Arc::clone(&stop_signal);
					let keyboard = Arc::clone(&keyboard);
					let speed_choice = Arc::from(Mutex::from(speed_choice.clone()));
					let thread_ended_signal = Arc::clone(&thread_ended_signal);
					thread::spawn(move || {
						thread_ended_signal.store(false, Ordering::Relaxed);
						while !stop_signal.load(Ordering::Relaxed) {
							let zone = rand::thread_rng().gen_range(0..4);
							let steps = rand::thread_rng().gen_range(50..=200);
							keyboard.lock().set_zone_by_index(zone, [255.0; 3]);
							// keyboard.lock().set_colors_to(&[255.0; 12]);
							keyboard
								.lock()
								.transition_colors_to(&[0.0; 12], steps / speed_choice.lock().choice().unwrap().parse::<u8>().unwrap(), 5);
							let sleep_time = rand::thread_rng().gen_range(100..=2000);
							thread::sleep(Duration::from_millis(sleep_time));
						}
						thread_ended_signal.store(true, Ordering::Relaxed);
					});
				}
				"AmbientLight" => {
					//Preparations
					stop_signal.store(true, Ordering::Relaxed);
					wait_thread_end(&thread_ended_signal);
					control_tiles.deactivate();
					stop_signal.store(false, Ordering::Relaxed);
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Static);

					//Create necessary clones to be passed into thread
					let stop_signal = Arc::clone(&stop_signal);
					let keyboard = Arc::clone(&keyboard);
					let thread_ended_signal = Arc::clone(&thread_ended_signal);
					thread::spawn(move || {
						thread_ended_signal.store(false, Ordering::Relaxed);

						//Display setup
						let displays = Display::all().unwrap().len();
						for i in 0..displays {
							let display = Display::all().unwrap().remove(i);
							if display.width() == DISP_WIDTH as usize && display.height() == DISP_HEIGHT as usize {
								let mut capturer = Capturer::new(display, false).expect("Couldn't begin capture.");
								let (w, h) = (capturer.width(), capturer.height());

								let seconds_per_frame = Duration::from_nanos(1_000_000_000 / 30);
								type BgraImage<V> = image::ImageBuffer<image::Bgra<u8>, V>;
								while !stop_signal.load(Ordering::Relaxed) {
									if let Ok(frame) = capturer.frame(0) {
										let now = Instant::now();
										let bgra_img = BgraImage::from_raw(w as u32, h as u32, &*frame).expect("Could not get bgra image.");
										let rgb_img: image::RgbImage = bgra_img.convert();
										let resized = image::imageops::resize(&rgb_img, 4, 1, image::imageops::FilterType::Lanczos3);
										let dst = resized.into_vec();

										let mut result: [f32; 12] = [0.0; 12];
										for i in 0..12 {
											result[i] = dst[i] as f32;
										}
										keyboard.lock().transition_colors_to(&result, 4, 1);
										let elapsed_time = now.elapsed();
										if elapsed_time < seconds_per_frame {
											thread::sleep(seconds_per_frame - elapsed_time);
										}
									} else {
										//Janky recover from error because it does not like admin prompts on windows
										let displays = Display::all().unwrap().len();
										for i in 0..displays {
											let display = Display::all().unwrap().remove(i);
											if display.width() == DISP_WIDTH as usize && display.height() == DISP_HEIGHT as usize {
												capturer = Capturer::new(display, false).expect("Couldn't begin capture.");
											}
										}
									}
									thread::sleep(Duration::from_millis(20));
								}
								thread_ended_signal.store(true, Ordering::Relaxed);
								drop(capturer);
							}
						}
					});
				}
				"SmoothLeftWave" => {
					//Preparations
					stop_signal.store(true, Ordering::Relaxed);
					wait_thread_end(&thread_ended_signal);
					control_tiles.deactivate();
					stop_signal.store(false, Ordering::Relaxed);
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Static);

					//Create necessary clones to be passed into thread
					let stop_signal = Arc::clone(&stop_signal);
					let keyboard = Arc::clone(&keyboard);
					let speed_choice = Arc::from(Mutex::from(speed_choice.clone()));
					let thread_ended_signal = Arc::clone(&thread_ended_signal);

					thread::spawn(move || {
						let mut gradient = vec![255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 255.0, 255.0, 0.0, 255.0];
						thread_ended_signal.store(false, Ordering::Relaxed);
						while !stop_signal.load(Ordering::Relaxed) {
							shift_vec(&mut gradient, 3);
							let colors: [f32; 12] = gradient.clone().try_into().unwrap();
							keyboard.lock().transition_colors_to(&colors, 70 / speed_choice.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							if stop_signal.load(Ordering::Relaxed) {
								break;
							}
							thread::sleep(Duration::from_millis(20));
						}
						thread_ended_signal.store(true, Ordering::Relaxed);
					});
				}
				"SmoothRightWave" => {
					//Preparations
					stop_signal.store(true, Ordering::Relaxed);
					wait_thread_end(&thread_ended_signal);
					control_tiles.deactivate();
					stop_signal.store(false, Ordering::Relaxed);
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Static);

					//Create necessary clones to be passed into thread
					let stop_signal = Arc::clone(&stop_signal);
					let keyboard = Arc::clone(&keyboard);
					let speed_choice = Arc::from(Mutex::from(speed_choice.clone()));
					let thread_ended_signal = Arc::clone(&thread_ended_signal);

					thread::spawn(move || {
						let mut gradient = vec![255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 255.0, 255.0, 0.0, 255.0];
						thread_ended_signal.store(false, Ordering::Relaxed);
						while !stop_signal.load(Ordering::Relaxed) {
							shift_vec(&mut gradient, 9);
							let colors: [f32; 12] = gradient.clone().try_into().unwrap();
							keyboard.lock().transition_colors_to(&colors, 70 / speed_choice.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							if stop_signal.load(Ordering::Relaxed) {
								break;
							}
							thread::sleep(Duration::from_millis(20));
						}
						thread_ended_signal.store(true, Ordering::Relaxed);
					});
				}
				"LeftSwipe" => {
					//Preparations
					stop_signal.store(true, Ordering::Relaxed);
					wait_thread_end(&thread_ended_signal);
					control_tiles.activate();
					control_tiles.master.deactivate();
					stop_signal.store(false, Ordering::Relaxed);
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Static);

					//Create necessary clones to be passed into thread
					let stop_signal = Arc::clone(&stop_signal);
					let keyboard = Arc::clone(&keyboard);
					let speed_choice = Arc::from(Mutex::from(speed_choice.clone()));
					let thread_ended_signal = Arc::clone(&thread_ended_signal);
					let control_tiles = Arc::from(Mutex::from(control_tiles.clone()));
					thread::spawn(move || {
						thread_ended_signal.store(false, Ordering::Relaxed);
						while !stop_signal.load(Ordering::Relaxed) {
							let zones_lock = &control_tiles.lock().zones;
							let mut gradient = vec![
								zones_lock.left.red_input.value().parse::<f32>().unwrap(),
								zones_lock.left.green_input.value().parse::<f32>().unwrap(),
								zones_lock.left.blue_input.value().parse::<f32>().unwrap(),
								zones_lock.center_left.red_input.value().parse::<f32>().unwrap(),
								zones_lock.center_left.green_input.value().parse::<f32>().unwrap(),
								zones_lock.center_left.blue_input.value().parse::<f32>().unwrap(),
								zones_lock.center_right.red_input.value().parse::<f32>().unwrap(),
								zones_lock.center_right.green_input.value().parse::<f32>().unwrap(),
								zones_lock.center_right.blue_input.value().parse::<f32>().unwrap(),
								zones_lock.right.red_input.value().parse::<f32>().unwrap(),
								zones_lock.right.green_input.value().parse::<f32>().unwrap(),
								zones_lock.right.blue_input.value().parse::<f32>().unwrap(),
							];
							for _i in 0..4 {
								shift_vec(&mut gradient, 3);
								let colors: [f32; 12] = gradient.clone().try_into().unwrap();
								keyboard.lock().transition_colors_to(&colors, 150 / speed_choice.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
								if stop_signal.load(Ordering::Relaxed) {
									break;
								}
							}
							if stop_signal.load(Ordering::Relaxed) {
								break;
							}
							thread::sleep(Duration::from_millis(20));
						}
						thread_ended_signal.store(true, Ordering::Relaxed);
					});
				}
				"RightSwipe" => {
					//Preparations
					stop_signal.store(true, Ordering::Relaxed);
					wait_thread_end(&thread_ended_signal);
					control_tiles.activate();
					control_tiles.master.deactivate();
					stop_signal.store(false, Ordering::Relaxed);
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Static);

					//Create necessary clones to be passed into thread
					let stop_signal = Arc::clone(&stop_signal);
					let keyboard = Arc::clone(&keyboard);
					let speed_choice = Arc::from(Mutex::from(speed_choice.clone()));
					let thread_ended_signal = Arc::clone(&thread_ended_signal);
					let control_tiles = Arc::from(Mutex::from(control_tiles.clone()));
					thread::spawn(move || {
						thread_ended_signal.store(false, Ordering::Relaxed);
						while !stop_signal.load(Ordering::Relaxed) {
							let zones_lock = &control_tiles.lock().zones;
							let mut gradient = vec![
								zones_lock.left.red_input.value().parse::<f32>().unwrap(),
								zones_lock.left.green_input.value().parse::<f32>().unwrap(),
								zones_lock.left.blue_input.value().parse::<f32>().unwrap(),
								zones_lock.center_left.red_input.value().parse::<f32>().unwrap(),
								zones_lock.center_left.green_input.value().parse::<f32>().unwrap(),
								zones_lock.center_left.blue_input.value().parse::<f32>().unwrap(),
								zones_lock.center_right.red_input.value().parse::<f32>().unwrap(),
								zones_lock.center_right.green_input.value().parse::<f32>().unwrap(),
								zones_lock.center_right.blue_input.value().parse::<f32>().unwrap(),
								zones_lock.right.red_input.value().parse::<f32>().unwrap(),
								zones_lock.right.green_input.value().parse::<f32>().unwrap(),
								zones_lock.right.blue_input.value().parse::<f32>().unwrap(),
							];
							for _i in 0..4 {
								shift_vec(&mut gradient, 9);
								let colors: [f32; 12] = gradient.clone().try_into().unwrap();
								keyboard.lock().transition_colors_to(&colors, 150 / speed_choice.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
								if stop_signal.load(Ordering::Relaxed) {
									break;
								}
							}
							if stop_signal.load(Ordering::Relaxed) {
								break;
							}
							thread::sleep(Duration::from_millis(20));
						}
						thread_ended_signal.store(true, Ordering::Relaxed);
					});
				}
				_ => {}
			},
		}
	});

	//Speed
	speed_choice.set_callback({
		let keyboard = keyboard.clone();
		move |choice| {
			if let Some(value) = choice.choice() {
				let speed = value.parse::<u8>().unwrap();
				if (1..=4).contains(&speed) {
					keyboard.lock().set_speed(speed);
				}
			}
		}
	});

	//Brightness
	brightness_choice.set_callback({
		move |choice| {
			if let Some(value) = choice.choice() {
				let brightness = value.parse::<u8>().unwrap();
				if (1..=2).contains(&brightness) {
					keyboard.lock().set_brightness(brightness);
				}
			}
		}
	});
	win
}

fn create_control_tiles(keyboard: Arc<Mutex<crate::keyboard_utils::Keyboard>>, stop_signal: Arc<AtomicBool>) -> ControlTiles {
	fn new_zone_control_tile(master_tile: bool) -> ZoneControlTile {
		let center_x = 540 / 2;
		let center_y = 90 / 2 - 20;
		let offset = 120;
		//Begin tile
		let mut control_tile = ZoneControlTile {
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

	fn add_zone_control_tile_handle(control_tile: &mut ZoneControlTile, keyboard: Arc<Mutex<crate::keyboard_utils::Keyboard>>, zone_index: u8, stop_signal: Arc<AtomicBool>) {
		//Button
		control_tile.toggle_button.handle({
			let keyboard = keyboard.clone();
			let mut control_tile = control_tile.clone();
			move |button, event| match event {
				Event::Released => {
					match button.is_toggled() {
						true => {
							keyboard.lock().set_zone_by_index(zone_index, [0.0; 3]);
							control_tile.red_input.deactivate();
							control_tile.green_input.deactivate();
							control_tile.blue_input.deactivate();
						}
						false => {
							keyboard.lock().set_zone_by_index(
								zone_index,
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
		fn add_input_handle(input: &mut IntInput, color: BaseColor, keyboard: Arc<Mutex<crate::keyboard_utils::Keyboard>>, zone_index: u8, stop_signal: Arc<AtomicBool>) {
			let triplet_index = zone_index * 3;
			let color_index = match color {
				BaseColor::Red => 0,
				BaseColor::Green => 1,
				BaseColor::Blue => 2,
			};
			input.handle({
				let keyboard = keyboard;
				let stop_signal = Arc::clone(&stop_signal);
				move |input, event| match event {
					Event::KeyUp => {
						match input.value().parse::<f32>() {
							Ok(val) => {
								input.set_value(&val.to_string());
								if val > 255.0 {
									input.set_value("255");
									if stop_signal.load(Ordering::Relaxed) {
										keyboard.lock().set_value_by_index(triplet_index + color_index, 255.0);
									}
								} else {
									if stop_signal.load(Ordering::Relaxed) {
										keyboard.lock().set_value_by_index(triplet_index + color_index, val);
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
		add_input_handle(&mut control_tile.red_input, BaseColor::Red, keyboard.clone(), zone_index, stop_signal.clone());
		//Green
		add_input_handle(&mut control_tile.green_input, BaseColor::Green, keyboard.clone(), zone_index, stop_signal.clone());
		//Blue
		add_input_handle(&mut control_tile.blue_input, BaseColor::Blue, keyboard, zone_index, stop_signal);
	}

	fn add_master_control_tile_handle(control_tiles: &mut ControlTiles, keyboard: Arc<Mutex<crate::keyboard_utils::Keyboard>>, stop_signal: Arc<AtomicBool>) {
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
							control_tiles.zones.deactivate();
						}
						false => {
							force_update_colors(&control_tiles.zones, &keyboard);
							master_tile.red_input.activate();
							master_tile.green_input.activate();
							master_tile.blue_input.activate();
							control_tiles.zones.activate();
						}
					}
					true
				}
				_ => false,
			}
		});
		fn add_master_input_handle(input: &mut IntInput, color: BaseColor, keyboard: Arc<Mutex<crate::keyboard_utils::Keyboard>>, control_tiles: ControlTiles, stop_signal: Arc<AtomicBool>) {
			let index = match color {
				BaseColor::Red => 0,
				BaseColor::Green => 1,
				BaseColor::Blue => 2,
			};
			input.handle({
				let keyboard = keyboard;
				let mut control_tiles = control_tiles;
				let stop_signal = Arc::clone(&stop_signal);
				move |input, event| match event {
					Event::KeyUp => {
						match input.value().parse::<f32>() {
							Ok(val) => {
								input.set_value(&val.to_string());
								if val > 255.0 {
									input.set_value("255");
									if stop_signal.load(Ordering::Relaxed) {
										keyboard.lock().solid_set_value_by_index(index, 255.0);
									}
									control_tiles.zones.change_color_value(color, 255.0);
								} else {
									if stop_signal.load(Ordering::Relaxed) {
										keyboard.lock().solid_set_value_by_index(index, val);
									}
									control_tiles.zones.change_color_value(color, val);
								}
							}
							Err(_) => {
								input.set_value("0");
								control_tiles.zones.change_color_value(color, 0.0);
							}
						}
						true
					}
					_ => false,
				}
			});
		}
		//Red
		add_master_input_handle(&mut master_tile.red_input, BaseColor::Red, keyboard.clone(), control_tiles.clone(), stop_signal.clone());
		//Green
		add_master_input_handle(&mut master_tile.green_input, BaseColor::Green, keyboard.clone(), control_tiles.clone(), stop_signal.clone());
		//Blue
		add_master_input_handle(&mut master_tile.blue_input, BaseColor::Blue, keyboard, control_tiles.clone(), stop_signal);
	}

	let mut zones = KeyboardZoneTiles {
		left: (new_zone_control_tile(false)),
		center_left: (new_zone_control_tile(false)),
		center_right: (new_zone_control_tile(false)),
		right: (new_zone_control_tile(false)),
	};

	add_zone_control_tile_handle(&mut zones.left, keyboard.clone(), 0, stop_signal.clone());
	add_zone_control_tile_handle(&mut zones.center_left, keyboard.clone(), 1, stop_signal.clone());
	add_zone_control_tile_handle(&mut zones.center_right, keyboard.clone(), 2, stop_signal.clone());
	add_zone_control_tile_handle(&mut zones.right, keyboard.clone(), 3, stop_signal.clone());

	let control_tiles = ControlTiles {
		master: (new_zone_control_tile(true)),
		zones,
	};

	add_master_control_tile_handle(&mut control_tiles.clone(), keyboard.clone(), stop_signal.clone());

	control_tiles
}

fn force_update_colors(zones: &KeyboardZoneTiles, keyboard: &Arc<Mutex<crate::keyboard_utils::Keyboard>>) {
	let target = [
		zones.left.red_input.value().parse::<f32>().unwrap(),
		zones.left.green_input.value().parse::<f32>().unwrap(),
		zones.left.blue_input.value().parse::<f32>().unwrap(),
		zones.center_left.red_input.value().parse::<f32>().unwrap(),
		zones.center_left.green_input.value().parse::<f32>().unwrap(),
		zones.center_left.blue_input.value().parse::<f32>().unwrap(),
		zones.center_right.red_input.value().parse::<f32>().unwrap(),
		zones.center_right.green_input.value().parse::<f32>().unwrap(),
		zones.center_right.blue_input.value().parse::<f32>().unwrap(),
		zones.right.red_input.value().parse::<f32>().unwrap(),
		zones.right.green_input.value().parse::<f32>().unwrap(),
		zones.right.blue_input.value().parse::<f32>().unwrap(),
	];
	keyboard.lock().set_colors_to(&target);
}

fn wait_thread_end(thread_end_signal: &Arc<AtomicBool>) {
	while !thread_end_signal.load(Ordering::Relaxed) {
		thread::sleep(Duration::from_millis(100));
	}
}

fn shift_vec(vec: &mut Vec<f32>, steps: u8) {
	for _i in 0..steps {
		let temp = vec.pop().unwrap();
		vec.insert(0, temp);
	}
}
