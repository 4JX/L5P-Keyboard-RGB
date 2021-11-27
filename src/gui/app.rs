use super::color_tiles::{ColorTiles, ColorTilesState};
use super::options::OptionsTile;
use super::{color_tiles, effect_browser, options};
use crate::gui::menu_bar;
use crate::keyboard_manager::{self, StopSignals};
use crate::{
	enums::{Effects, Message},
	gui::dialog::{alert, panic},
};
use fltk::enums::FrameType;
use fltk::{app, enums::Font, prelude::*, window::Window};
use fltk::{
	browser::HoldBrowser,
	dialog,
	prelude::{BrowserExt, MenuExt},
	text,
};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::time::Duration;
use std::{panic, thread};
use std::{path, str::FromStr, sync::mpsc::Sender};

const WIDTH: i32 = 900;
const HEIGHT: i32 = 480;

pub fn screen_center() -> (i32, i32) {
	((app::screen_size().0 / 2.0) as i32, (app::screen_size().1 / 2.0) as i32)
}

#[cfg(target_os = "windows")]
pub const EFFECTS_LIST: [&str; 14] = [
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
	"Fade",
];

#[cfg(target_os = "linux")]
pub const EFFECTS_LIST: [&str; 15] = [
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
	"Fade",
	"Temperature",
];

#[derive(Serialize, Deserialize)]
struct Profile {
	pub color_tiles_state: ColorTilesState,
	pub effect: Effects,
	pub speed: i32,
	pub brightness: i32,
}
#[derive(Clone)]
pub struct App {
	pub color_tiles: ColorTiles,
	pub effect_browser: HoldBrowser,
	pub options_tile: OptionsTile,
	pub tx: Sender<Message>,
	pub stop_signals: StopSignals,
	pub buf: text::TextBuffer,
	pub center: (i32, i32),
}

impl App {
	pub fn load_profile(&mut self, is_default: bool) {
		let filename: String;
		if is_default {
			filename = String::from_str("default.json").unwrap();
		} else {
			let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseFile);
			dlg.set_option(dialog::FileDialogOptions::NoOptions);
			dlg.set_filter("*.json");
			dlg.show();
			filename = dlg.filename().to_string_lossy().to_string();
		}

		if filename.is_empty() {
			self.stop_signals.store_true();
			self.tx.send(Message::Refresh).unwrap();
		} else {
			fn parse_profile(profile_text: &str) -> Result<Profile> {
				serde_json::from_str(profile_text)
			}

			if path::Path::new(&filename).exists() {
				self.buf.load_file(&filename).unwrap();

				if let Ok(profile) = parse_profile(&self.buf.text()) {
					self.color_tiles.set_state(&profile.color_tiles_state, profile.effect);
					self.effect_browser.select(EFFECTS_LIST.iter().position(|&val| val == profile.effect.to_string()).unwrap() as i32 + 1);
					self.options_tile.speed_choice.set_value(profile.speed);
					self.options_tile.brightness_choice.set_value(profile.brightness);
					self.stop_signals.store_true();
					self.tx.send(Message::UpdateEffect { effect: profile.effect }).unwrap();
				} else {
					alert(
						800,
						200,
						"There was an error loading the profile.\nPlease make sure its a valid profile file and that it is compatible with this version of the program.",
					);
					self.stop_signals.store_true();
					self.tx.send(Message::Refresh).unwrap();
				}
			} else if !is_default {
				alert(800, 200, "File does not exist!");
			}
		}
	}

	pub fn save_profile(&mut self) {
		let profile = Profile {
			color_tiles_state: self.color_tiles.get_state(),
			effect: Effects::from_str(self.effect_browser.selected_text().unwrap().as_str()).unwrap(),
			speed: self.options_tile.speed_choice.value(),
			brightness: self.options_tile.brightness_choice.value(),
		};

		self.buf.set_text(serde_json::to_string(&profile).unwrap().as_str());

		let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseSaveFile);
		dlg.set_option(dialog::FileDialogOptions::SaveAsConfirm);
		dlg.show();
		let mut filename = dlg.filename().to_string_lossy().to_string();
		if !filename.is_empty() {
			if !filename.rsplit('.').next().map(|ext| ext.eq_ignore_ascii_case("json")).unwrap() {
				filename = format!("{}{}", &filename, ".json");
			}
			self.buf.save_file(filename).unwrap_or_else(|_| alert(800, 200, "Please specify a file name to use."));
		}

		self.stop_signals.store_true();
		self.tx.send(Message::Refresh).unwrap();
	}

	pub fn start_ui(mut manager: keyboard_manager::KeyboardManager) -> fltk::window::Window {
		panic::set_hook(Box::new(|info| {
			if let Some(s) = info.payload().downcast_ref::<&str>() {
				panic(800, 400, s);
			} else {
				panic(800, 400, &info.to_string());
			}
		}));

		//UI
		let mut win = Window::new(screen_center().0 - WIDTH / 2, screen_center().1 - HEIGHT / 2, WIDTH, HEIGHT, "Legion Keyboard RGB Control");
		let tiles = color_tiles::ColorTiles::new(0, 30, &manager.tx, manager.stop_signals.clone());

		let mut app = Self {
			color_tiles: tiles,
			effect_browser: effect_browser::EffectBrowserTile::create(540, 30, &EFFECTS_LIST).effect_browser,
			options_tile: options::OptionsTile::create(540, 390, &manager.tx, &manager.stop_signals),
			tx: manager.tx.clone(),
			stop_signals: manager.stop_signals.clone(),
			buf: text::TextBuffer::default(),
			center: screen_center(),
		};

		menu_bar::AppMenuBar::new(&app);

		win.end();
		win.make_resizable(false);
		win.show();

		// Theming
		app::background(51, 51, 51);
		app::background2(119, 119, 119);
		app::foreground(0, 0, 0);
		app::set_visible_focus(false);
		app::set_font(Font::HelveticaBold);
		app::set_frame_border_radius_max(5);
		app::set_frame_type(FrameType::FlatBox);
		app::set_frame_type2(FrameType::DownBox, FrameType::RoundedBox);

		app.load_profile(true);

		// Effect choice
		app.effect_browser.set_callback({
			let stop_signals = app.stop_signals.clone();
			let tx = app.tx.clone();
			move |browser| {
				stop_signals.store_true();
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
					13 => {
						tx.send(Message::UpdateEffect { effect: Effects::Christmas }).unwrap();
					}
					14 => {
						tx.send(Message::UpdateEffect { effect: Effects::Fade }).unwrap();
					}
					15 => {
						tx.send(Message::UpdateEffect { effect: Effects::Temperature }).unwrap();
					}
					_ => unreachable!("Effect index is out of range"),
				}
			}
		});

		thread::spawn(move || loop {
			match manager.rx.try_iter().last() {
				Some(message) => {
					match message {
						Message::UpdateEffect { effect } => {
							app.color_tiles.update(effect);
							app::awake();
							let color_array = app.color_tiles.get_zone_values();
							let speed = app.options_tile.speed_choice.choice().unwrap().parse::<u8>().unwrap();
							let brightness = app.options_tile.brightness_choice.choice().unwrap().parse::<u8>().unwrap();
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
							app.tx.send(Message::Refresh).unwrap();
						}
						Message::UpdateSpeed { speed } => {
							manager.keyboard.set_speed(speed);
							app.tx.send(Message::Refresh).unwrap();
						}
						Message::Refresh => {
							app.tx.send(Message::UpdateEffect { effect: manager.last_effect }).unwrap();
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
}
