use super::enums::Message;
use super::keyboard_manager;
use super::{effect_browser_tile, enums::BaseColor, enums::Effects, keyboard_color_tiles, options_tile};
use fltk::{
	app,
	enums::{Event, Font},
	group::Pack,
	input::IntInput,
	prelude::*,
	window::Window,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const WIDTH: i32 = 900;
const HEIGHT: i32 = 450;

pub fn start_ui(mut manager: keyboard_manager::KeyboardManager, tx: mpsc::Sender<Message>, stop_signal: &Arc<AtomicBool>) -> fltk::window::Window {
	//UI
	let mut win = Window::default().with_size(WIDTH, HEIGHT).with_label("Legion Keyboard RGB Control");
	let mut color_picker_pack = Pack::new(0, 0, 540, 360, "");
	let mut keyboard_color_tiles = create_keyboard_color_tiles(&tx, stop_signal.clone());

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
		"Disco",
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
	//Speed
	speed_choice.set_callback({
		let tx = tx.clone();
		let stop_signal = stop_signal.clone();
		move |choice| {
			stop_signal.store(true, Ordering::Relaxed);
			if let Some(value) = choice.choice() {
				let speed = value.parse::<u8>().unwrap();
				if (1..=4).contains(&speed) {
					tx.send(Message::UpdateSpeed { speed }).unwrap();
				}
			}
		}
	});

	//Brightness
	brightness_choice.set_callback({
		let tx = tx.clone();
		let stop_signal = stop_signal.clone();
		move |choice| {
			stop_signal.store(true, Ordering::Relaxed);
			if let Some(value) = choice.choice() {
				let brightness = value.parse::<u8>().unwrap();
				if (1..=2).contains(&brightness) {
					tx.send(Message::UpdateBrightness { brightness }).unwrap();
				}
			}
		}
	});

	// Effect choice
	effect_browser.set_callback({
		let tx = tx.clone();
		let stop_signal = stop_signal.clone();
		move |browser| {
			stop_signal.store(true, Ordering::Relaxed);
			match browser.value() {
				0 => {
					browser.select(0);
				}
				1 => {
					tx.send(Message::UpdateEffect { effect: Effects::Static }).unwrap();
				}
				2 => {
					tx.send(Message::UpdateEffect { effect: Effects::Breath }).unwrap();
				}
				3 => {
					tx.send(Message::UpdateEffect { effect: Effects::Smooth }).unwrap();
				}
				4 => {
					tx.send(Message::UpdateEffect { effect: Effects::LeftWave }).unwrap();
				}
				5 => {
					tx.send(Message::UpdateEffect { effect: Effects::RightWave }).unwrap();
				}
				6 => {
					tx.send(Message::UpdateEffect { effect: Effects::Lightning }).unwrap();
				}
				7 => {
					tx.send(Message::UpdateEffect { effect: Effects::AmbientLight }).unwrap();
				}
				8 => {
					tx.send(Message::UpdateEffect { effect: Effects::SmoothLeftWave }).unwrap();
				}
				9 => {
					tx.send(Message::UpdateEffect { effect: Effects::SmoothRightWave }).unwrap();
				}
				10 => {
					tx.send(Message::UpdateEffect { effect: Effects::LeftSwipe }).unwrap();
				}
				11 => {
					tx.send(Message::UpdateEffect { effect: Effects::RightSwipe }).unwrap();
				}
				12 => {
					tx.send(Message::UpdateEffect { effect: Effects::Disco }).unwrap();
				}
				_ => {}
			}
		}
	});

	thread::spawn(move || loop {
		if let Ok(message) = manager.rx.recv() {
			match message {
				Message::UpdateEffect { effect } => {
					let color_array = keyboard_color_tiles.zones.get_values();
					let speed = speed_choice.choice().unwrap().parse::<u8>().unwrap();
					manager.set_effect(effect, &color_array, speed);
				}
				Message::UpdateAllValues { value } => {
					manager.keyboard.set_colors_to(&value);
				}
				Message::UpdateRGB { index, value } => {
					manager.keyboard.solid_set_value_by_index(index, value);
				}
				Message::UpdateZone { zone_index, value } => {
					manager.keyboard.set_zone_by_index(zone_index, value);
				}
				Message::UpdateValue { index, value } => {
					manager.keyboard.set_value_by_index(index, value);
				}
				Message::UpdateBrightness { brightness } => {
					manager.keyboard.set_brightness(brightness);
					tx.send(Message::Restart).unwrap();
				}
				Message::UpdateSpeed { speed } => {
					manager.keyboard.set_speed(speed);
					tx.send(Message::Restart).unwrap();
				}
				Message::Restart => {
					tx.send(Message::UpdateEffect { effect: manager.last_effect }).unwrap();
				}
			}
			app::awake();
			thread::sleep(Duration::from_millis(20));
		}
	});
	win
}

fn create_keyboard_color_tiles(tx: &mpsc::Sender<Message>, stop_signal: Arc<AtomicBool>) -> keyboard_color_tiles::KeyboardColorTiles {
	fn add_zone_tile_handle(color_tile: &mut keyboard_color_tiles::ColorTile, tx: &mpsc::Sender<Message>, zone_index: u8, stop_signal: Arc<AtomicBool>) {
		fn add_input_handle(input: &mut IntInput, color: BaseColor, tx: mpsc::Sender<Message>, zone_index: u8, stop_signal: Arc<AtomicBool>) {
			let triplet_index = zone_index * 3;
			let color_index = match color {
				BaseColor::Red => 0,
				BaseColor::Green => 1,
				BaseColor::Blue => 2,
			};
			let index = triplet_index + color_index;
			input.handle({
				move |input, event| match event {
					Event::KeyUp => {
						match input.value().parse::<f32>() {
							Ok(value) => {
								input.set_value(&value.to_string());
								if value > 255.0 {
									input.set_value("255");
								}
								if !stop_signal.load(Ordering::Relaxed) {
									tx.send(Message::UpdateValue {
										index,
										value: input.value().parse().unwrap(),
									})
									.unwrap();
								}
								stop_signal.store(true, Ordering::Relaxed);
								tx.send(Message::Restart).unwrap();
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
		color_tile.toggle_button.handle({
			let mut color_tile = color_tile.clone();
			let tx = tx.clone();
			let stop_signal = stop_signal.clone();
			move |button, event| match event {
				Event::Released => {
					if button.is_toggled() {
						tx.send(Message::UpdateZone { zone_index, value: [0.0; 3] }).unwrap();
						color_tile.red_input.deactivate();
						color_tile.green_input.deactivate();
						color_tile.blue_input.deactivate();
					} else {
						tx.send(Message::UpdateZone {
							zone_index,
							value: color_tile.get_values(),
						})
						.unwrap();
						color_tile.red_input.activate();
						color_tile.green_input.activate();
						color_tile.blue_input.activate();
					}
					stop_signal.store(true, Ordering::Relaxed);
					tx.send(Message::Restart).unwrap();
					true
				}
				_ => false,
			}
		});
		//Red
		add_input_handle(&mut color_tile.red_input, BaseColor::Red, tx.clone(), zone_index, stop_signal.clone());
		//Green
		add_input_handle(&mut color_tile.green_input, BaseColor::Green, tx.clone(), zone_index, stop_signal.clone());
		//Blue
		add_input_handle(&mut color_tile.blue_input, BaseColor::Blue, tx.clone(), zone_index, stop_signal);
	}

	fn add_master_tile_handle(keyboard_color_tiles: &mut keyboard_color_tiles::KeyboardColorTiles, tx: &mpsc::Sender<Message>, stop_signal: Arc<AtomicBool>) {
		fn add_master_input_handle(input: &mut IntInput, color: BaseColor, tx: mpsc::Sender<Message>, keyboard_color_tiles: keyboard_color_tiles::KeyboardColorTiles, stop_signal: Arc<AtomicBool>) {
			let index = match color {
				BaseColor::Red => 0,
				BaseColor::Green => 1,
				BaseColor::Blue => 2,
			};
			input.handle({
				let mut keyboard_color_tiles = keyboard_color_tiles;
				move |input, event| match event {
					Event::KeyUp => {
						if let Ok(value) = input.value().parse::<f32>() {
							input.set_value(&value.to_string());
							if value > 255.0 {
								input.set_value("255");
							}
							if !stop_signal.load(Ordering::Relaxed) {
								tx.send(Message::UpdateRGB {
									index,
									value: input.value().parse().unwrap(),
								})
								.unwrap();
								keyboard_color_tiles.zones.change_color_value(color, input.value().parse().unwrap());
							}
							stop_signal.store(true, Ordering::Relaxed);
							tx.send(Message::Restart).unwrap();
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
			let mut keyboard_color_tiles = keyboard_color_tiles.clone();
			let mut master_tile = master_tile.clone();
			let tx = tx.clone();
			let stop_signal = stop_signal.clone();
			move |button, event| match event {
				Event::Released => {
					if button.is_toggled() {
						tx.send(Message::UpdateAllValues { value: [255.0; 12] }).unwrap();
						master_tile.red_input.deactivate();
						master_tile.green_input.deactivate();
						master_tile.blue_input.deactivate();
						keyboard_color_tiles.zones.deactivate();
					} else {
						tx.send(Message::UpdateAllValues {
							value: keyboard_color_tiles.zones.get_values(),
						})
						.unwrap();
						master_tile.red_input.activate();
						master_tile.green_input.activate();
						master_tile.blue_input.activate();
						keyboard_color_tiles.zones.activate();
					}
					stop_signal.store(true, Ordering::Relaxed);
					tx.send(Message::Restart).unwrap();
					true
				}
				_ => false,
			}
		});
		//Red
		add_master_input_handle(&mut master_tile.red_input, BaseColor::Red, tx.clone(), keyboard_color_tiles.clone(), stop_signal.clone());
		//Green
		add_master_input_handle(&mut master_tile.green_input, BaseColor::Green, tx.clone(), keyboard_color_tiles.clone(), stop_signal.clone());
		//Blue
		add_master_input_handle(&mut master_tile.blue_input, BaseColor::Blue, tx.clone(), keyboard_color_tiles.clone(), stop_signal);
	}

	let mut keyboard_color_tiles = keyboard_color_tiles::KeyboardColorTiles {
		master: (keyboard_color_tiles::ColorTile::create(true)),
		zones: keyboard_color_tiles::ZoneColorTiles::create(),
	};

	add_zone_tile_handle(&mut keyboard_color_tiles.zones.left, tx, 0, stop_signal.clone());
	add_zone_tile_handle(&mut keyboard_color_tiles.zones.center_left, tx, 1, stop_signal.clone());
	add_zone_tile_handle(&mut keyboard_color_tiles.zones.center_right, tx, 2, stop_signal.clone());
	add_zone_tile_handle(&mut keyboard_color_tiles.zones.right, tx, 3, stop_signal.clone());
	add_master_tile_handle(&mut keyboard_color_tiles.clone(), tx, stop_signal);

	keyboard_color_tiles
}
