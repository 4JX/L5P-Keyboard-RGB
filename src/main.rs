#![windows_subsystem = "windows"]
use std::sync::Arc;
use std::{thread, time};

use fltk::{enums::Event, input::IntInput, prelude::*};

use parking_lot::Mutex;
use rand::Rng;

mod gui;
mod keyboard_utils;

fn add_control_tile_handle(control_tile: &mut gui::RgbControlTile, keyboard: Arc<Mutex<keyboard_utils::Keyboard>>, index: u8, effect_loop_is_active: Arc<Mutex<bool>>) {
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

	fn add_input_handle(input: &mut IntInput, color: gui::BaseColor, keyboard: Arc<Mutex<keyboard_utils::Keyboard>>, triplet_index: u8, effect_loop_is_active: Arc<Mutex<bool>>) {
		let index = match color {
			gui::BaseColor::Red => 0,
			gui::BaseColor::Green => 1,
			gui::BaseColor::Blue => 2,
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
	add_input_handle(&mut control_tile.red_input, gui::BaseColor::Red, keyboard.clone(), index, effect_loop_is_active.clone());
	//Green
	add_input_handle(&mut control_tile.green_input, gui::BaseColor::Green, keyboard.clone(), index, effect_loop_is_active.clone());
	//Blue
	add_input_handle(&mut control_tile.blue_input, gui::BaseColor::Blue, keyboard, index, effect_loop_is_active);
}

fn add_master_control_tile_handle(control_tiles: &mut gui::ControlTiles, keyboard: Arc<Mutex<keyboard_utils::Keyboard>>, effect_loop_is_active: Arc<Mutex<bool>>) {
	let mut master_tile = control_tiles.master.1.clone();

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

	fn add_master_input_handle(input: &mut IntInput, color: gui::BaseColor, keyboard: Arc<Mutex<keyboard_utils::Keyboard>>, control_tiles: gui::ControlTiles, effect_loop_is_active: Arc<Mutex<bool>>) {
		let index = match color {
			gui::BaseColor::Red => 0,
			gui::BaseColor::Green => 1,
			gui::BaseColor::Blue => 2,
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
	add_master_input_handle(&mut master_tile.red_input, gui::BaseColor::Red, keyboard.clone(), control_tiles.clone(), effect_loop_is_active.clone());
	//Green
	add_master_input_handle(
		&mut master_tile.green_input,
		gui::BaseColor::Green,
		keyboard.clone(),
		control_tiles.clone(),
		effect_loop_is_active.clone(),
	);
	//Blue
	add_master_input_handle(&mut master_tile.blue_input, gui::BaseColor::Blue, keyboard, control_tiles.clone(), effect_loop_is_active);
}

fn force_update_colors(sections: &gui::SectionControlTiles, keyboard: &Arc<Mutex<keyboard_utils::Keyboard>>) {
	let target = [
		sections.left.1.red_input.value().parse::<f32>().unwrap(),
		sections.left.1.green_input.value().parse::<f32>().unwrap(),
		sections.left.1.blue_input.value().parse::<f32>().unwrap(),
		sections.center_left.1.red_input.value().parse::<f32>().unwrap(),
		sections.center_left.1.green_input.value().parse::<f32>().unwrap(),
		sections.center_left.1.blue_input.value().parse::<f32>().unwrap(),
		sections.center_right.1.red_input.value().parse::<f32>().unwrap(),
		sections.center_right.1.green_input.value().parse::<f32>().unwrap(),
		sections.center_right.1.blue_input.value().parse::<f32>().unwrap(),
		sections.right.1.red_input.value().parse::<f32>().unwrap(),
		sections.right.1.green_input.value().parse::<f32>().unwrap(),
		sections.right.1.blue_input.value().parse::<f32>().unwrap(),
	];
	keyboard.lock().set_colors_to(&target);
}

fn wait_thread_end(signal: &Arc<Mutex<bool>>) {
	while !*signal.lock() {
		thread::sleep(time::Duration::from_millis(100));
	}
}

fn main() {
	let keyboard: keyboard_utils::Keyboard = match keyboard_utils::get_keyboard() {
		Ok(keyboard) => keyboard,
		Err(err) => panic!("{}", err),
	};
	let keyboard = Arc::from(Mutex::from(keyboard));
	let effect_loop_is_active = Arc::new(Mutex::new(false));
	let thread_ended_signal = Arc::new(Mutex::new(true));

	//Begin app logic

	let mut app_ui = gui::create_ui();
	add_control_tile_handle(&mut app_ui.control_tiles.control_sections.left.1, keyboard.clone(), 0, effect_loop_is_active.clone());
	add_control_tile_handle(&mut app_ui.control_tiles.control_sections.center_left.1, keyboard.clone(), 1, effect_loop_is_active.clone());
	add_control_tile_handle(&mut app_ui.control_tiles.control_sections.center_right.1, keyboard.clone(), 2, effect_loop_is_active.clone());
	add_control_tile_handle(&mut app_ui.control_tiles.control_sections.right.1, keyboard.clone(), 3, effect_loop_is_active.clone());
	add_master_control_tile_handle(&mut app_ui.control_tiles.clone(), keyboard.clone(), effect_loop_is_active.clone());

	// Effect choice
	app_ui.effect_browser.set_callback({
		let keyboard = keyboard.clone();
		let mut app_ui = app_ui.clone();
		let thread_ended_signal = Arc::clone(&thread_ended_signal);
		move |browser| match browser.value() {
			0 => {
				browser.select(1);
			}
			_ => match app_ui.effects_list[(browser.value() - 1) as usize] {
				"Static" => {
					app_ui.control_tiles.activate();
					*effect_loop_is_active.lock() = false;
					keyboard.lock().set_effect(keyboard_utils::KeyboardEffects::Static);
					force_update_colors(&app_ui.control_tiles.control_sections, &keyboard);
				}
				"Breath" => {
					app_ui.control_tiles.activate();
					*effect_loop_is_active.lock() = false;
					keyboard.lock().set_effect(keyboard_utils::KeyboardEffects::Breath);
					force_update_colors(&app_ui.control_tiles.control_sections, &keyboard);
				}
				"Smooth" => {
					app_ui.control_tiles.deactivate();
					*effect_loop_is_active.lock() = false;
					keyboard.lock().set_effect(keyboard_utils::KeyboardEffects::Smooth);
				}
				"LeftWave" => {
					app_ui.control_tiles.deactivate();
					*effect_loop_is_active.lock() = false;
					keyboard.lock().set_effect(keyboard_utils::KeyboardEffects::LeftWave);
				}
				"RightWave" => {
					app_ui.control_tiles.deactivate();
					*effect_loop_is_active.lock() = false;

					keyboard.lock().set_effect(keyboard_utils::KeyboardEffects::RightWave);
				}
				"LeftPulse" => {
					//Preparations
					*effect_loop_is_active.lock() = false;
					wait_thread_end(&thread_ended_signal);
					app_ui.control_tiles.master_only();
					*effect_loop_is_active.lock() = true;
					keyboard.lock().set_effect(keyboard_utils::KeyboardEffects::Static);

					//Create necessary clones to be passed into thread
					let should_loop = Arc::clone(&effect_loop_is_active);
					let keyboard = Arc::clone(&keyboard);
					let speed = Arc::from(Mutex::from(app_ui.speed.clone()));
					let control_tiles = Arc::from(Mutex::from(app_ui.control_tiles.clone()));
					let signal = Arc::clone(&thread_ended_signal);
					thread::spawn(move || {
						*signal.lock() = false;
						while *should_loop.lock() {
							let master_tile = &control_tiles.lock().master.1;
							let red = master_tile.red_input.value().parse::<f32>().unwrap();
							let green = master_tile.green_input.value().parse::<f32>().unwrap();
							let blue = master_tile.blue_input.value().parse::<f32>().unwrap();

							let color: [f32; 12] = [red, green, blue, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
							keyboard.lock().transition_colors_to(&color, 255 / speed.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							if !*should_loop.lock() {
								break;
							}
							let color: [f32; 12] = [0.0, 0.0, 0.0, red, green, blue, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
							keyboard.lock().transition_colors_to(&color, 255 / speed.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							if !*should_loop.lock() {
								break;
							}
							let color: [f32; 12] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, red, green, blue, 0.0, 0.0, 0.0];
							keyboard.lock().transition_colors_to(&color, 255 / speed.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							if !*should_loop.lock() {
								break;
							}
							let color: [f32; 12] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, red, green, blue];
							keyboard.lock().transition_colors_to(&color, 255 / speed.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							thread::sleep(time::Duration::from_millis(50));
						}
						*signal.lock() = true;
					});
				}
				"RightPulse" => {
					//Preparations
					*effect_loop_is_active.lock() = false;
					wait_thread_end(&thread_ended_signal);
					app_ui.control_tiles.master_only();
					*effect_loop_is_active.lock() = true;
					keyboard.lock().set_effect(keyboard_utils::KeyboardEffects::Static);

					//Create necessary clones to be passed into thread
					let should_loop = Arc::clone(&effect_loop_is_active);
					let keyboard = Arc::clone(&keyboard);
					let speed = Arc::from(Mutex::from(app_ui.speed.clone()));
					let control_tiles = Arc::from(Mutex::from(app_ui.control_tiles.clone()));
					let signal = Arc::clone(&thread_ended_signal);
					thread::spawn(move || {
						*signal.lock() = false;
						while *should_loop.lock() {
							let master_tile = &control_tiles.lock().master.1;
							let red = master_tile.red_input.value().parse::<f32>().unwrap();
							let green = master_tile.green_input.value().parse::<f32>().unwrap();
							let blue = master_tile.blue_input.value().parse::<f32>().unwrap();
							let color: [f32; 12] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, red, green, blue];
							keyboard.lock().transition_colors_to(&color, 255 / speed.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							if !*should_loop.lock() {
								break;
							}
							let color: [f32; 12] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, red, green, blue, 0.0, 0.0, 0.0];
							keyboard.lock().transition_colors_to(&color, 255 / speed.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							if !*should_loop.lock() {
								break;
							}
							let color: [f32; 12] = [0.0, 0.0, 0.0, red, green, blue, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
							keyboard.lock().transition_colors_to(&color, 255 / speed.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							if !*should_loop.lock() {
								break;
							}
							let color: [f32; 12] = [red, green, blue, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
							keyboard.lock().transition_colors_to(&color, 255 / speed.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							thread::sleep(time::Duration::from_millis(50));
						}
						*signal.lock() = true;
					});
				}
				"Lightning" => {
					//Preparations
					*effect_loop_is_active.lock() = false;
					wait_thread_end(&thread_ended_signal);
					app_ui.control_tiles.deactivate();
					*effect_loop_is_active.lock() = true;
					keyboard.lock().set_effect(keyboard_utils::KeyboardEffects::Static);

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
	app_ui.speed.set_callback({
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
	app_ui.brightness.set_callback({
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

	app_ui.app.run().unwrap();
}
