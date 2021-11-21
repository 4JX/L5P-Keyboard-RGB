use super::{color_tiles, effect_browser_tile, options_tile};
use crate::enums::{Effects, Message};
use crate::gui::app::App;
use crate::gui::{dialog, menu_bar};
use crate::keyboard_manager;
use fltk::enums::FrameType;
use fltk::text;
use fltk::{app, enums::Font, group::Pack, prelude::*, window::Window};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;
use std::{panic, thread};

const WIDTH: i32 = 900;
const HEIGHT: i32 = 480;
pub const EFFECTS_LIST: [&str; 13] = [
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
	"Christmas",
];

pub fn screen_center() -> (i32, i32) {
	((app::screen_size().0 / 2.0) as i32, (app::screen_size().1 / 2.0) as i32)
}

pub fn start_ui(mut manager: keyboard_manager::KeyboardManager, tx: mpsc::Sender<Message>, stop_signal: &Arc<AtomicBool>) -> fltk::window::Window {
	panic::set_hook(Box::new(|info| {
		if let Some(s) = info.payload().downcast_ref::<&str>() {
			dialog::panic(800, 400, s);
		} else {
			dialog::panic(800, 400, &info.to_string());
		}
	}));

	//UI
	let mut win = Window::new(screen_center().0 - WIDTH / 2, screen_center().1 - HEIGHT / 2, WIDTH, HEIGHT, "Legion Keyboard RGB Control");
	let mut color_picker_pack = Pack::new(0, 30, 540, 360, "");
	let mut tiles = color_tiles::ColorTiles::new(&tx, stop_signal.clone());

	color_picker_pack.add(&tiles.zones.left.exterior_tile);
	color_picker_pack.add(&tiles.zones.center_left.exterior_tile);
	color_picker_pack.add(&tiles.zones.center_right.exterior_tile);
	color_picker_pack.add(&tiles.zones.right.exterior_tile);
	color_picker_pack.add(&tiles.master.exterior_tile);
	color_picker_pack.end();

	let effect_browser_tile = effect_browser_tile::EffectBrowserTile::create(540, 30, &EFFECTS_LIST);
	let mut effect_browser = effect_browser_tile.effect_browser;

	let options_tile = options_tile::OptionsTile::create(540, 390, &tx, &stop_signal.clone());
	let speed_choice = options_tile.speed_choice;
	let brightness_choice = options_tile.brightness_choice;

	let mut app = App {
		color_tiles: tiles.clone(),
		effect_browser: effect_browser.clone(),
		speed_choice: speed_choice.clone(),
		brightness_choice: brightness_choice.clone(),
		tx: tx.clone(),
		stop_signal: stop_signal.clone(),
		buf: text::TextBuffer::default(),
		center: screen_center(),
	};

	menu_bar::AppMenuBar::new(&tx, stop_signal.clone(), &app);

	win.end();
	win.make_resizable(false);
	win.show();

	// Theming
	app::background(51, 51, 51);
	app::set_visible_focus(false);
	app::set_font(Font::HelveticaBold);
	app::set_frame_type(FrameType::FlatBox);

	// Effect choice
	effect_browser.set_callback({
		let stop_signal = stop_signal.clone();
		let tx = tx.clone();
		let mut color_tiles = tiles.clone();
		move |browser| {
			stop_signal.store(true, Ordering::SeqCst);
			match browser.value() {
				0 => {
					browser.select(0);
				}
				1 => {
					color_tiles.activate();
					tx.send(Message::UpdateEffect { effect: Effects::Static }).unwrap();
				}
				2 => {
					color_tiles.activate();
					tx.send(Message::UpdateEffect { effect: Effects::Breath }).unwrap();
				}
				3 => {
					color_tiles.deactivate();
					tx.send(Message::UpdateEffect { effect: Effects::Smooth }).unwrap();
				}
				4 => {
					color_tiles.deactivate();
					tx.send(Message::UpdateEffect { effect: Effects::LeftWave }).unwrap();
				}
				5 => {
					color_tiles.deactivate();
					tx.send(Message::UpdateEffect { effect: Effects::RightWave }).unwrap();
				}
				6 => {
					color_tiles.deactivate();
					tx.send(Message::UpdateEffect { effect: Effects::Lightning }).unwrap();
				}
				7 => {
					color_tiles.deactivate();
					tx.send(Message::UpdateEffect { effect: Effects::AmbientLight }).unwrap();
				}
				8 => {
					color_tiles.deactivate();
					tx.send(Message::UpdateEffect { effect: Effects::SmoothLeftWave }).unwrap();
				}
				9 => {
					color_tiles.deactivate();
					tx.send(Message::UpdateEffect { effect: Effects::SmoothRightWave }).unwrap();
				}
				10 => {
					color_tiles.activate();
					tx.send(Message::UpdateEffect { effect: Effects::LeftSwipe }).unwrap();
				}
				11 => {
					color_tiles.activate();
					tx.send(Message::UpdateEffect { effect: Effects::RightSwipe }).unwrap();
				}
				12 => {
					color_tiles.deactivate();
					tx.send(Message::UpdateEffect { effect: Effects::Disco }).unwrap();
				}
				13 => {
					color_tiles.deactivate();
					tx.send(Message::UpdateEffect { effect: Effects::Christmas }).unwrap();
				}
				_ => {}
			}
		}
	});

	app.load_profile(true);

	thread::spawn(move || loop {
		match manager.rx.try_iter().last() {
			Some(message) => {
				match message {
					Message::UpdateEffect { effect } => {
						let color_array = tiles.get_zone_values();
						let speed = speed_choice.choice().unwrap().parse::<u8>().unwrap();
						let brightness = brightness_choice.choice().unwrap().parse::<u8>().unwrap();
						manager.set_effect(effect, &color_array, speed, brightness);
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
					Message::SaveProfile => {
						app.save_profile();
					}
					Message::LoadProfile => {
						app.load_profile(false);
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
