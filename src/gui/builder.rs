use super::{
	effect_browser_tile,
	enums::{BaseColor, Effects},
	keyboard_color_tiles, keyboard_effect_manager, options_tile,
};
use crate::keyboard_utils;
use fltk::{
	app,
	enums::{Event, Font},
	group::Pack,
	input::IntInput,
	prelude::*,
	window::Window,
};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const WIDTH: i32 = 900;
const HEIGHT: i32 = 450;

pub fn start_ui(keyboard: Arc<Mutex<keyboard_utils::Keyboard>>) -> fltk::window::Window {
	//UI
	let mut win = Window::default().with_size(WIDTH, HEIGHT).with_label("Legion Keyboard RGB Control");
	let mut color_picker_pack = Pack::new(0, 0, 540, 360, "");
	let keyboard_color_tiles = create_keyboard_color_tiles(keyboard.clone(), &keyboard.lock().stop_signal.clone());

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
	let effect_browser_tile = effect_browser_tile::EffectBrowserTile::create(&effects_list);
	let mut effect_browser = effect_browser_tile.effect_browser;

	let options_tile = options_tile::OptionsTile::create();
	let mut speed_choice = options_tile.speed_choice;
	let mut brightness_choice = options_tile.brightness_choice;

	win.end();
	win.make_resizable(false);
	win.show();

	// Theming
	app::background(51, 51, 51);
	app::set_visible_focus(false);
	app::set_font(Font::HelveticaBold);

	//Begin app logic
	// Effect choice
	let mut effect_manager = keyboard_effect_manager::EffectManager {
		keyboard: keyboard.clone(),
		keyboard_color_tiles: keyboard_color_tiles.clone(),
		speed_choice: speed_choice.clone(),
		thread_ended_signal: Arc::new(AtomicBool::new(true)),
	};

	effect_browser.set_callback({
		move |browser| match browser.value() {
			0 => {
				browser.select(0);
			}

			1 => {
				effect_manager.change_effect(Effects::Static);
			}
			2 => {
				effect_manager.change_effect(Effects::Breath);
			}
			3 => {
				effect_manager.change_effect(Effects::Smooth);
			}
			4 => {
				effect_manager.change_effect(Effects::LeftWave);
			}
			5 => {
				effect_manager.change_effect(Effects::RightWave);
			}
			6 => {
				effect_manager.change_effect(Effects::Lightning);
			}
			7 => {
				effect_manager.change_effect(Effects::AmbientLight);
			}
			8 => {
				effect_manager.change_effect(Effects::SmoothLeftWave);
			}
			9 => {
				effect_manager.change_effect(Effects::SmoothRightWave);
			}
			10 => {
				effect_manager.change_effect(Effects::LeftSwipe);
			}
			11 => {
				effect_manager.change_effect(Effects::RightSwipe);
			}
			_ => {}
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

fn create_keyboard_color_tiles(keyboard: Arc<Mutex<keyboard_utils::Keyboard>>, stop_signal: &Arc<AtomicBool>) -> keyboard_color_tiles::KeyboardColorTiles {
	fn add_zone_tile_handle(control_tile: &mut keyboard_color_tiles::ColorTile, keyboard: Arc<Mutex<keyboard_utils::Keyboard>>, zone_index: u8, stop_signal: &Arc<AtomicBool>) {
		fn add_input_handle(input: &mut IntInput, color: BaseColor, keyboard: Arc<Mutex<keyboard_utils::Keyboard>>, zone_index: u8, stop_signal: &Arc<AtomicBool>) {
			let triplet_index = zone_index * 3;
			let color_index = match color {
				BaseColor::Red => 0,
				BaseColor::Green => 1,
				BaseColor::Blue => 2,
			};
			input.handle({
				let keyboard = keyboard;
				let stop_signal = Arc::clone(stop_signal);
				move |input, event| match event {
					Event::KeyUp => {
						match input.value().parse::<f32>() {
							Ok(val) => {
								input.set_value(&val.to_string());
								if val > 255.0 {
									input.set_value("255");
									if !stop_signal.load(Ordering::Relaxed) {
										keyboard.lock().set_value_by_index(triplet_index + color_index, 255.0);
									}
								} else if !stop_signal.load(Ordering::Relaxed) {
									keyboard.lock().set_value_by_index(triplet_index + color_index, val);
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
		//Button
		control_tile.toggle_button.handle({
			let keyboard = keyboard.clone();
			let mut control_tile = control_tile.clone();
			move |button, event| match event {
				Event::Released => {
					if button.is_toggled() {
						keyboard.lock().set_zone_by_index(zone_index, [0.0; 3]);
						control_tile.red_input.deactivate();
						control_tile.green_input.deactivate();
						control_tile.blue_input.deactivate();
					} else {
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
					true
				}
				_ => false,
			}
		});
		//Red
		add_input_handle(&mut control_tile.red_input, BaseColor::Red, keyboard.clone(), zone_index, &stop_signal.clone());
		//Green
		add_input_handle(&mut control_tile.green_input, BaseColor::Green, keyboard.clone(), zone_index, &stop_signal.clone());
		//Blue
		add_input_handle(&mut control_tile.blue_input, BaseColor::Blue, keyboard, zone_index, stop_signal);
	}

	fn add_master_tile_handle(keyboard_color_tiles: &mut keyboard_color_tiles::KeyboardColorTiles, keyboard: Arc<Mutex<keyboard_utils::Keyboard>>, stop_signal: &Arc<AtomicBool>) {
		fn add_master_input_handle(
			input: &mut IntInput, color: BaseColor, keyboard: Arc<Mutex<keyboard_utils::Keyboard>>, keyboard_color_tiles: keyboard_color_tiles::KeyboardColorTiles, stop_signal: &Arc<AtomicBool>,
		) {
			let index = match color {
				BaseColor::Red => 0,
				BaseColor::Green => 1,
				BaseColor::Blue => 2,
			};
			input.handle({
				let keyboard = keyboard;
				let mut keyboard_color_tiles = keyboard_color_tiles;
				let stop_signal = Arc::clone(stop_signal);
				move |input, event| match event {
					Event::KeyUp => {
						if let Ok(val) = input.value().parse::<f32>() {
							input.set_value(&val.to_string());
							if val > 255.0 {
								input.set_value("255");
								if !stop_signal.load(Ordering::Relaxed) {
									keyboard.lock().solid_set_value_by_index(index, 255.0);
								}
								keyboard_color_tiles.zones.change_color_value(color, 255.0);
							} else if !stop_signal.load(Ordering::Relaxed) {
								keyboard.lock().solid_set_value_by_index(index, val);
								keyboard_color_tiles.zones.change_color_value(color, val);
							}
						} else {
							input.set_value("0");
							keyboard_color_tiles.zones.change_color_value(color, 0.0);
						}
						true
					}
					_ => false,
				}
			});
		}
		let mut master_tile = keyboard_color_tiles.master.clone();
		//Button
		master_tile.toggle_button.handle({
			let keyboard = keyboard.clone();
			let mut keyboard_color_tiles = keyboard_color_tiles.clone();
			let mut master_tile = master_tile.clone();
			move |button, event| match event {
				Event::Released => {
					if button.is_toggled() {
						keyboard.lock().set_colors_to(&[0.0; 12]);
						master_tile.red_input.deactivate();
						master_tile.green_input.deactivate();
						master_tile.blue_input.deactivate();
						keyboard_color_tiles.zones.deactivate();
					} else {
						force_update_colors(&keyboard_color_tiles.zones, &keyboard);
						master_tile.red_input.activate();
						master_tile.green_input.activate();
						master_tile.blue_input.activate();
						keyboard_color_tiles.zones.activate();
					}
					true
				}
				_ => false,
			}
		});
		//Red
		add_master_input_handle(&mut master_tile.red_input, BaseColor::Red, keyboard.clone(), keyboard_color_tiles.clone(), &stop_signal.clone());
		//Green
		add_master_input_handle(&mut master_tile.green_input, BaseColor::Green, keyboard.clone(), keyboard_color_tiles.clone(), &stop_signal.clone());
		//Blue
		add_master_input_handle(&mut master_tile.blue_input, BaseColor::Blue, keyboard, keyboard_color_tiles.clone(), stop_signal);
	}

	let mut keyboard_color_tiles = keyboard_color_tiles::KeyboardColorTiles {
		master: (keyboard_color_tiles::ColorTile::create(true)),
		zones: keyboard_color_tiles::ZoneColorTiles::create(),
	};

	add_zone_tile_handle(&mut keyboard_color_tiles.zones.left, keyboard.clone(), 0, stop_signal);
	add_zone_tile_handle(&mut keyboard_color_tiles.zones.center_left, keyboard.clone(), 1, stop_signal);
	add_zone_tile_handle(&mut keyboard_color_tiles.zones.center_right, keyboard.clone(), 2, stop_signal);
	add_zone_tile_handle(&mut keyboard_color_tiles.zones.right, keyboard.clone(), 3, stop_signal);
	add_master_tile_handle(&mut keyboard_color_tiles.clone(), keyboard, stop_signal);

	keyboard_color_tiles
}

pub fn force_update_colors(zones: &keyboard_color_tiles::ZoneColorTiles, keyboard: &Arc<Mutex<keyboard_utils::Keyboard>>) {
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
