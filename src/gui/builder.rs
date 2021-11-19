use super::{color_tiles, effect_browser_tile, options_tile};
use crate::enums::{Effects, Message};
use crate::gui::app::App;
use crate::gui::menu_bar;
use crate::keyboard_manager;
use fltk::enums::FrameType;
use fltk::{app, enums::Font, group::Pack, prelude::*, window::Window};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const WIDTH: i32 = 900;
const HEIGHT: i32 = 480;

pub fn start_ui(mut manager: keyboard_manager::KeyboardManager, tx: mpsc::Sender<Message>, stop_signal: &Arc<AtomicBool>) -> fltk::window::Window {
	//UI
	let mut win = Window::default().with_size(WIDTH, HEIGHT).with_label("Legion Keyboard RGB Control");
	menu_bar::AppMenuBar::new(&tx);
	let mut color_picker_pack = Pack::new(0, 30, 540, 360, "");
	let mut tiles = color_tiles::ColorTiles::new(&tx, stop_signal.clone());

	color_picker_pack.add(&tiles.zones.left.exterior_tile);
	color_picker_pack.add(&tiles.zones.center_left.exterior_tile);
	color_picker_pack.add(&tiles.zones.center_right.exterior_tile);
	color_picker_pack.add(&tiles.zones.right.exterior_tile);
	color_picker_pack.add(&tiles.master.exterior_tile);
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
	let effect_browser_tile = effect_browser_tile::EffectBrowserTile::create(540, 30, &effects_list);
	let mut effect_browser = effect_browser_tile.effect_browser;

	let options_tile = options_tile::OptionsTile::create(540, 390, &tx, &stop_signal.clone());
	let speed_choice = options_tile.speed_choice;
	let brightness_choice = options_tile.brightness_choice;

	win.end();
	win.make_resizable(false);
	win.show();

	// Theming
	app::background(51, 51, 51);
	app::set_visible_focus(false);
	app::set_font(Font::HelveticaBold);
	app::set_frame_type(FrameType::FlatBox);

	let mut app = App {
		color_tiles: tiles.clone(),
		effect_browser: effect_browser.clone(),
		speed_choice: speed_choice.clone(),
		brightness_choice,
		tx: tx.clone(),
		stop_signal: stop_signal.clone(),
	};

	// Effect choice
	effect_browser.set_callback({
		let stop_signal = stop_signal.clone();

		move |browser| {
			stop_signal.store(true, Ordering::SeqCst);
			match browser.value() {
				0 => {
					browser.select(0);
				}
				1 => {
					app.set_effect(Effects::Static);
				}
				2 => {
					app.set_effect(Effects::Breath);
				}
				3 => {
					app.set_effect(Effects::Smooth);
				}
				4 => {
					app.set_effect(Effects::LeftWave);
				}
				5 => {
					app.set_effect(Effects::RightWave);
				}
				6 => {
					app.set_effect(Effects::Lightning);
				}
				7 => {
					app.set_effect(Effects::AmbientLight);
				}
				8 => {
					app.set_effect(Effects::SmoothLeftWave);
				}
				9 => {
					app.set_effect(Effects::SmoothRightWave);
				}
				10 => {
					app.set_effect(Effects::LeftSwipe);
				}
				11 => {
					app.set_effect(Effects::RightSwipe);
				}
				12 => {
					app.set_effect(Effects::Disco);
				}
				_ => {}
			}
		}
	});

	thread::spawn(move || loop {
		match manager.rx.try_iter().last() {
			Some(message) => {
				match message {
					Message::UpdateEffect { effect } => {
						let color_array = tiles.get_zone_values();
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
						tx.send(Message::Refresh).unwrap();
					}
					Message::UpdateSpeed { speed } => {
						manager.keyboard.set_speed(speed);
						tx.send(Message::Refresh).unwrap();
					}
					Message::Refresh => {
						tx.send(Message::UpdateEffect { effect: manager.last_effect }).unwrap();
					}
				}
				app::awake();
			}
			None => {
				thread::sleep(Duration::from_millis(20));
			}
		}
	});
	win
}
