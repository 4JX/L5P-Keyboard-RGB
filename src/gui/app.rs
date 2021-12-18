use super::color_tiles::ColorTiles;
use super::options::OptionsTile;
use super::utils::screen_center;
use super::{color_tiles, effect_browser, options};
use crate::gui::dialog as appdialog;
use crate::gui::menu_bar;
use crate::keyboard_manager::{KeyboardManager, StopSignals};
use crate::profile::Profile;
use crate::{
	custom_effect::CustomEffect,
	enums::{Direction, Effects, Message},
};
use clap::crate_name;
use fltk::browser::HoldBrowser;
use fltk::dialog;
use fltk::enums::FrameType;
use fltk::{app, enums::Font, prelude::*, window::Window};
use flume::Sender;
use single_instance::SingleInstance;
use std::convert::TryInto;
use std::str::FromStr;
use std::time::Duration;
use std::{panic, path, thread};

const WIDTH: i32 = 900;
const HEIGHT: i32 = 570;

#[derive(Clone)]
pub struct App {
	pub color_tiles: ColorTiles,
	pub effect_browser: HoldBrowser,
	pub options_tile: OptionsTile,
	pub tx: Sender<Message>,
	pub stop_signals: StopSignals,
	pub center: (i32, i32),
}

impl App {
	pub fn start_ui() {
		let app = app::App::default();

		app::background(51, 51, 51);
		app::background2(119, 119, 119);
		app::foreground(0, 0, 0);
		app::set_visible_focus(false);
		app::set_font(Font::HelveticaBold);
		app::set_frame_border_radius_max(5);
		app::set_frame_type(FrameType::FlatBox);
		app::set_frame_type2(FrameType::DownBox, FrameType::RoundedBox);

		let instance = SingleInstance::new(crate_name!()).unwrap();
		if !instance.is_single() {
			println!("Not single");
			appdialog::alert(800, 400, "Another instance of the program is already running, please close it before starting a new one.", true);
			app.run().unwrap();
		}

		let manager = KeyboardManager::new().unwrap();

		//Windows logic
		#[cfg(target_os = "windows")]
		{
			use fltk::prelude::*;
			use tray_item::{IconSource, TrayItem};

			type HWND = *mut std::os::raw::c_void;

			static mut WINDOW: HWND = std::ptr::null_mut();

			let mut win = Self::create_window(manager);

			unsafe {
				WINDOW = win.raw_handle();
			}
			win.set_callback(|_| {
				extern "C" {
					pub fn ShowWindow(hwnd: HWND, nCmdShow: i32) -> bool;
				}
				unsafe {
					ShowWindow(WINDOW, 0);
				}
			});
			//Create tray icon
			let mut tray = TrayItem::new("Keyboard RGB", IconSource::Resource("trayIcon")).unwrap();

			tray.add_menu_item("Show", move || {
				extern "C" {
					pub fn ShowWindow(hwnd: HWND, nCmdShow: i32) -> bool;
				}
				unsafe {
					ShowWindow(WINDOW, 9);
				}
			})
			.unwrap();

			tray.add_menu_item("Quit", || {
				println!("Quit");
				std::process::exit(0);
			})
			.unwrap();

			loop {
				if win.shown() {
					app.run().unwrap();
				} else {
					app::sleep(0.05);
				}
			}
		}

		#[cfg(target_os = "linux")]
		{
			Self::create_window(manager);
			app.run().unwrap();
		}
	}

	pub fn load_profile(&mut self, is_default: bool) {
		let filename = if is_default {
			"default.json".to_string()
		} else {
			let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseFile);
			dlg.set_option(dialog::FileDialogOptions::NoOptions);
			dlg.set_filter("*.json");
			dlg.show();
			dlg.filename().to_string_lossy().to_string()
		};

		if filename.is_empty() {
			self.stop_signals.store_true();
			self.tx.send(Message::Refresh).unwrap();
		} else if path::Path::new(&filename).exists() {
			if let Ok(profile) = Profile::from_file(filename) {
				self.color_tiles.set_state(&profile.rgb_array, profile.ui_toggle_button_state, profile.effect);
				self.effect_browser.select(profile.effect as i32 + 1);
				self.options_tile.speed_choice.set_value(profile.speed.into());
				self.options_tile.brightness_choice.set_value(profile.brightness.into());

				self.stop_signals.store_true();
				self.tx.send(Message::UpdateEffect { effect: profile.effect }).unwrap();
			} else {
				appdialog::alert(
					800,
					200,
					"There was an error loading the profile.\nPlease make sure its a valid profile file and that it is compatible with this version of the program.",
					false,
				);
				self.stop_signals.store_true();
				self.tx.send(Message::Refresh).unwrap();
			}
		} else if !is_default {
			appdialog::alert(800, 200, "File does not exist!", false);
		}
	}

	pub fn save_profile(&mut self) {
		let rgb_array = self.color_tiles.get_values();
		let effect = Effects::from_str(self.effect_browser.selected_text().unwrap().as_str()).unwrap();
		let direction = Direction::from_str(self.options_tile.direction_choice.choice().unwrap().as_str()).unwrap();
		let speed = self.options_tile.speed_choice.value();
		let brightness = self.options_tile.brightness_choice.value();

		let profile = Profile::new(rgb_array, effect, direction, speed.try_into().unwrap(), brightness.try_into().unwrap(), [false; 5]);

		let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseSaveFile);
		dlg.set_option(dialog::FileDialogOptions::SaveAsConfirm);
		dlg.show();

		let filename = dlg.filename().to_string_lossy().to_string();

		if !filename.is_empty() {
			profile.save(filename.as_str()).unwrap();
		}

		self.stop_signals.store_true();
		self.tx.send(Message::Refresh).unwrap();
	}

	pub fn load_custom_profile(&mut self) {
		let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseFile);
		dlg.set_option(dialog::FileDialogOptions::NoOptions);
		dlg.set_filter("*.json");
		dlg.show();
		let filename = dlg.filename().to_string_lossy().to_string();

		if filename.is_empty() {
			self.stop_signals.store_true();
			self.tx.send(Message::Refresh).unwrap();
		} else if path::Path::new(&filename).exists() {
			if let Ok(effect) = CustomEffect::from_file(filename) {
				self.stop_signals.store_true();
				self.tx.send(Message::CustomEffect { effect }).unwrap();
			} else {
				appdialog::alert(
					800,
					200,
					"There was an error loading the custom effect.\nPlease make sure its a valid custom effect file and that it is compatible with this version of the program.",
					false,
				);
				self.stop_signals.store_true();
				self.tx.send(Message::Refresh).unwrap();
			}
		} else {
			appdialog::alert(800, 200, "File does not exist!", false);
		}
	}

	pub fn create_window(mut manager: KeyboardManager) -> fltk::window::Window {
		panic::set_hook(Box::new(|info| {
			if let Some(s) = info.payload().downcast_ref::<&str>() {
				appdialog::panic(800, 400, s);
			} else {
				appdialog::panic(800, 400, &info.to_string());
			}
		}));

		let mut win = Window::new(screen_center().0 - WIDTH / 2, screen_center().1 - HEIGHT / 2, WIDTH, HEIGHT, "Legion Keyboard RGB Control");

		let mut app = Self {
			color_tiles: color_tiles::ColorTiles::new(0, 30, &manager.tx, &manager.stop_signals),
			effect_browser: effect_browser::EffectBrowserTile::create(540, 30, &manager.tx, &manager.stop_signals).effect_browser,
			options_tile: options::OptionsTile::create(0, 480, &manager.tx, &manager.stop_signals),
			tx: manager.tx.clone(),
			stop_signals: manager.stop_signals.clone(),
			center: screen_center(),
		};

		menu_bar::AppMenuBar::new(&app);

		let icon_str = include_str!("../../res/trayIcon.svg");
		let icon_svg = fltk::image::SvgImage::from_data(icon_str).unwrap();
		win.set_icon(Some(icon_svg));
		win.end();
		win.make_resizable(false);
		win.show();

		app.update(Effects::Static);
		app.load_profile(true);

		thread::spawn(move || loop {
			match manager.rx.try_iter().last() {
				Some(message) => {
					match message {
						Message::UpdateEffect { effect } => {
							app.update(effect);
							app::awake();
							let color_array = app.color_tiles.get_values();
							let speed = app.options_tile.speed_choice.choice().unwrap().parse::<u8>().unwrap();
							let brightness = app.options_tile.brightness_choice.choice().unwrap().parse::<u8>().unwrap();
							let direction = Direction::from_str(app.options_tile.direction_choice.choice().unwrap().as_str()).unwrap();

							manager.set_effect(effect, direction, &color_array, speed, brightness);
						}
						Message::CustomEffect { effect } => {
							app.color_tiles.deactivate();
							effect.play(&mut manager);
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
	fn update(&mut self, effect: Effects) {
		self.color_tiles.update(effect);
		self.options_tile.update(effect);
	}
}
