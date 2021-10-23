use crate::gui::keyboard_color_tiles;

use fltk::{
	app,
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

const DARK_GRAY: u32 = 0x333333;

#[derive(Copy, Clone)]
pub enum BaseColor {
	Red,
	Green,
	Blue,
}

pub fn start_ui(keyboard: Arc<Mutex<crate::keyboard_utils::Keyboard>>) -> fltk::window::Window {
	//Keyboard
	let stop_signal = Arc::new(AtomicBool::new(true));
	let thread_ended_signal = Arc::new(AtomicBool::new(true));

	//UI
	let mut win = Window::default().with_size(WIDTH, HEIGHT).with_label("Legion Keyboard RGB Control");
	let mut color_picker_pack = Pack::new(0, 0, 540, 360, "");
	let mut keyboard_color_tiles = create_keyboard_color_tiles(keyboard.clone(), stop_signal.clone());

	color_picker_pack.add(&keyboard_color_tiles.zones.left.exterior_tile);
	color_picker_pack.add(&keyboard_color_tiles.zones.center_left.exterior_tile);
	color_picker_pack.add(&keyboard_color_tiles.zones.center_right.exterior_tile);
	color_picker_pack.add(&keyboard_color_tiles.zones.right.exterior_tile);
	color_picker_pack.add(&keyboard_color_tiles.master.exterior_tile);
	color_picker_pack.end();

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
	let mut effect_type_tile = Tile::new(540, 0, 360, 360, "");
	let mut effect_browser = crate::gui::effect_browser::EffectBrowser::new(&effects_list);
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
	//TODO: Move each custom effect to its own file in a directory
	//TODO: Also check out todo extentions
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
					keyboard_color_tiles.activate();
					stop_signal.store(true, Ordering::Relaxed);
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Static);
					force_update_colors(&keyboard_color_tiles.zones, &keyboard);
				}
				"Breath" => {
					keyboard_color_tiles.activate();
					stop_signal.store(true, Ordering::Relaxed);
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Breath);
					force_update_colors(&keyboard_color_tiles.zones, &keyboard);
				}
				"Smooth" => {
					keyboard_color_tiles.deactivate();
					stop_signal.store(true, Ordering::Relaxed);
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Smooth);
				}
				"LeftWave" => {
					keyboard_color_tiles.deactivate();
					stop_signal.store(true, Ordering::Relaxed);
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::LeftWave);
				}
				"RightWave" => {
					keyboard_color_tiles.deactivate();
					stop_signal.store(true, Ordering::Relaxed);

					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::RightWave);
				}
				"Lightning" => {
					//Preparations
					stop_signal.store(true, Ordering::Relaxed);
					wait_thread_end(&thread_ended_signal);
					keyboard_color_tiles.deactivate();
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
					keyboard_color_tiles.deactivate();
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
					keyboard_color_tiles.deactivate();
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
					keyboard_color_tiles.deactivate();
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
					keyboard_color_tiles.activate();
					keyboard_color_tiles.master.deactivate();
					stop_signal.store(false, Ordering::Relaxed);
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Static);

					//Create necessary clones to be passed into thread
					let stop_signal = Arc::clone(&stop_signal);
					let keyboard = Arc::clone(&keyboard);
					let speed_choice = Arc::from(Mutex::from(speed_choice.clone()));
					let thread_ended_signal = Arc::clone(&thread_ended_signal);
					let keyboard_color_tiles = Arc::from(Mutex::from(keyboard_color_tiles.clone()));
					thread::spawn(move || {
						thread_ended_signal.store(false, Ordering::Relaxed);
						while !stop_signal.load(Ordering::Relaxed) {
							let zones_lock = &keyboard_color_tiles.lock().zones;
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
					keyboard_color_tiles.activate();
					keyboard_color_tiles.master.deactivate();
					stop_signal.store(false, Ordering::Relaxed);
					keyboard.lock().set_effect(crate::keyboard_utils::LightingEffects::Static);

					//Create necessary clones to be passed into thread
					let stop_signal = Arc::clone(&stop_signal);
					let keyboard = Arc::clone(&keyboard);
					let speed_choice = Arc::from(Mutex::from(speed_choice.clone()));
					let thread_ended_signal = Arc::clone(&thread_ended_signal);
					let keyboard_color_tiles = Arc::from(Mutex::from(keyboard_color_tiles.clone()));
					thread::spawn(move || {
						thread_ended_signal.store(false, Ordering::Relaxed);
						while !stop_signal.load(Ordering::Relaxed) {
							let zones_lock = &keyboard_color_tiles.lock().zones;
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

fn create_keyboard_color_tiles(keyboard: Arc<Mutex<crate::keyboard_utils::Keyboard>>, stop_signal: Arc<AtomicBool>) -> keyboard_color_tiles::KeyboardColorTiles {
	fn add_zone_tile_handle(control_tile: &mut keyboard_color_tiles::ColorTile, keyboard: Arc<Mutex<crate::keyboard_utils::Keyboard>>, zone_index: u8, stop_signal: Arc<AtomicBool>) {
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
								if stop_signal.load(Ordering::Relaxed) {
									if val > 255.0 {
										println!("Val exceeded 255");
										input.set_value("255");
										keyboard.lock().set_value_by_index(triplet_index + color_index, 255.0);
									} else {
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

	fn add_master_control_tile_handle(keyboard_color_tiles: &mut keyboard_color_tiles::KeyboardColorTiles, keyboard: Arc<Mutex<crate::keyboard_utils::Keyboard>>, stop_signal: Arc<AtomicBool>) {
		let mut master_tile = keyboard_color_tiles.master.clone();
		//Button
		master_tile.toggle_button.handle({
			let keyboard = keyboard.clone();
			let mut keyboard_color_tiles = keyboard_color_tiles.clone();
			let mut master_tile = master_tile.clone();
			move |button, event| match event {
				Event::Released => {
					match button.is_toggled() {
						true => {
							keyboard.lock().set_colors_to(&[0.0; 12]);
							master_tile.red_input.deactivate();
							master_tile.green_input.deactivate();
							master_tile.blue_input.deactivate();
							keyboard_color_tiles.zones.deactivate();
						}
						false => {
							force_update_colors(&keyboard_color_tiles.zones, &keyboard);
							master_tile.red_input.activate();
							master_tile.green_input.activate();
							master_tile.blue_input.activate();
							keyboard_color_tiles.zones.activate();
						}
					}
					true
				}
				_ => false,
			}
		});
		fn add_master_input_handle(
			input: &mut IntInput, color: BaseColor, keyboard: Arc<Mutex<crate::keyboard_utils::Keyboard>>, keyboard_color_tiles: keyboard_color_tiles::KeyboardColorTiles, stop_signal: Arc<AtomicBool>,
		) {
			let index = match color {
				BaseColor::Red => 0,
				BaseColor::Green => 1,
				BaseColor::Blue => 2,
			};
			input.handle({
				let keyboard = keyboard;
				let mut keyboard_color_tiles = keyboard_color_tiles;
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
									keyboard_color_tiles.zones.change_color_value(color, 255.0);
								} else {
									if stop_signal.load(Ordering::Relaxed) {
										keyboard.lock().solid_set_value_by_index(index, val);
									}
									keyboard_color_tiles.zones.change_color_value(color, val);
								}
							}
							Err(_) => {
								input.set_value("0");
								keyboard_color_tiles.zones.change_color_value(color, 0.0);
							}
						}
						true
					}
					_ => false,
				}
			});
		}
		//Red
		add_master_input_handle(&mut master_tile.red_input, BaseColor::Red, keyboard.clone(), keyboard_color_tiles.clone(), stop_signal.clone());
		//Green
		add_master_input_handle(&mut master_tile.green_input, BaseColor::Green, keyboard.clone(), keyboard_color_tiles.clone(), stop_signal.clone());
		//Blue
		add_master_input_handle(&mut master_tile.blue_input, BaseColor::Blue, keyboard, keyboard_color_tiles.clone(), stop_signal);
	}

	let mut keyboard_color_tiles = keyboard_color_tiles::KeyboardColorTiles {
		master: (keyboard_color_tiles::ColorTile::new(true)),
		zones: keyboard_color_tiles::ZoneColorTiles::new(),
	};

	add_zone_tile_handle(&mut keyboard_color_tiles.zones.left, keyboard.clone(), 0, stop_signal.clone());
	add_zone_tile_handle(&mut keyboard_color_tiles.zones.center_left, keyboard.clone(), 1, stop_signal.clone());
	add_zone_tile_handle(&mut keyboard_color_tiles.zones.center_right, keyboard.clone(), 2, stop_signal.clone());
	add_zone_tile_handle(&mut keyboard_color_tiles.zones.right, keyboard.clone(), 3, stop_signal.clone());
	add_master_control_tile_handle(&mut keyboard_color_tiles.clone(), keyboard, stop_signal);

	keyboard_color_tiles
}

fn force_update_colors(zones: &keyboard_color_tiles::ZoneColorTiles, keyboard: &Arc<Mutex<crate::keyboard_utils::Keyboard>>) {
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
