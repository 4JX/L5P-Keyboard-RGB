use super::color_tiles::ColorTiles;
use super::options::OptionsTile;
use super::utils::screen_center;
use super::{color_tiles, options, side_tile};
use crate::gui::dialog as appdialog;
use crate::gui::menu_bar;
use crate::keyboard_manager::{KeyboardManager, StopSignals};
use crate::profile::Profile;
use crate::{
	custom_effect::CustomEffect,
	enums::{Direction, Effects, Message},
};
use clap::crate_name;
use device_query::{DeviceQuery, DeviceState, Keycode};
use fltk::browser::HoldBrowser;
use fltk::dialog;
use fltk::enums::FrameType;
use fltk::{app, enums::Font, prelude::*, window::Window};
use flume::Sender;
use single_instance::SingleInstance;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
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
	profile_vec: SharedVec<Profile>,
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
			appdialog::alert(800, 400, "Another instance of the program is already running, please close it before starting a new one.", true);
			app.run().unwrap();
		}

		let manager_result = KeyboardManager::new();
		if manager_result.is_err() {
			appdialog::alert(800, 400, "A valid keyboard model was not found. It may be due to a hardware error.", true);
			app.run().unwrap();
		}

		let manager = manager_result.unwrap();

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

	pub fn update_gui_from_profile(&mut self, profile: &Profile) {
		self.color_tiles.set_state(&profile.rgb_array, profile.ui_toggle_button_state);
		self.effect_browser.select(profile.effect as i32 + 1);
		self.options_tile.speed_choice.set_value(i32::from(profile.speed) - 1);
		self.options_tile.brightness_choice.set_value(i32::from(profile.brightness) - 1);

		self.stop_signals.store_true();
		self.tx.send(Message::Refresh).unwrap();
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
				self.update_gui_from_profile(&profile);
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

	pub fn create_profile_from_gui(&mut self) -> Profile {
		let rgb_array = self.color_tiles.get_values();
		let effect = Effects::from_str(self.effect_browser.selected_text().unwrap().as_str()).unwrap();
		let direction = Direction::from_str(self.options_tile.direction_choice.choice().unwrap().as_str()).unwrap();
		let speed = self.options_tile.speed_choice.choice().unwrap().parse::<u8>().unwrap();
		let brightness = self.options_tile.brightness_choice.choice().unwrap().parse::<u8>().unwrap();
		let ui_toggle_button_state = self.color_tiles.get_button_state();

		Profile::new(rgb_array, effect, direction, speed, brightness, ui_toggle_button_state)
	}

	pub fn save_profile(&mut self) {
		let profile = self.create_profile_from_gui();

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
		let mut side_tile = side_tile::SideTile::create(540, 35, &manager.tx, &manager.stop_signals);

		let mut app = Self {
			color_tiles: color_tiles::ColorTiles::new(0, 35, &manager.tx, &manager.stop_signals),
			effect_browser: side_tile.effect_browser,
			options_tile: options::OptionsTile::create(0, 480, &manager.tx, &manager.stop_signals),
			tx: manager.tx.clone(),
			stop_signals: manager.stop_signals.clone(),
			center: screen_center(),
			profile_vec: SharedVec::new(),
		};

		menu_bar::AppMenuBar::new(&app);

		fn update_preset_browser(preset_browser: &mut HoldBrowser, profile_vec: &SharedVec<Profile>) {
			preset_browser.clear();

			for i in 0..profile_vec.len() {
				preset_browser.add(format!("Preset {}", i).as_str());
			}
		}

		side_tile.add_preset_button.set_callback({
			let mut app = app.clone();
			let mut preset_browser = side_tile.preset_browser.clone();
			move |_button| {
				let profile = app.create_profile_from_gui();
				app.profile_vec.push(profile);
				update_preset_browser(&mut preset_browser, &app.profile_vec);
			}
		});

		side_tile.remove_preset_button.set_callback({
			let app = app.clone();
			let mut preset_browser = side_tile.preset_browser.clone();
			move |_button| {
				if preset_browser.value() > 0 && app.profile_vec.len() > 0 {
					app.profile_vec.remove(preset_browser.value() as usize - 1);
					update_preset_browser(&mut preset_browser, &app.profile_vec);
				}
			}
		});

		side_tile.preset_browser.set_callback({
			let mut app = app.clone();
			let preset_browser = side_tile.preset_browser.clone();
			move |_browser| {
				let profile_vec = app.profile_vec.inner.lock().unwrap().clone();
				if let Some(profile) = profile_vec.get(preset_browser.value() as usize - 1) {
					app.update_gui_from_profile(profile);
				};
			}
		});

		thread::spawn({
			let mut app = app.clone();
			let mut preset_browser = side_tile.preset_browser.clone();
			move || {
				let device_state = DeviceState::new();

				loop {
					let profile_vec = app.profile_vec.inner.lock().unwrap().clone();

					if !profile_vec.is_empty() {
						let keys: Vec<Keycode> = device_state.get_keys();

						if keys.contains(&Keycode::Meta) && keys.contains(&Keycode::RAlt) {
							if profile_vec.len() > 1 {
								if profile_vec.len() == preset_browser.value() as usize {
									preset_browser.select(1);
								} else {
									preset_browser.select(preset_browser.value() + 1);
								}
							} else {
								preset_browser.select(1);
							}

							if let Some(profile) = profile_vec.get(preset_browser.value() as usize - 1) {
								app.update_gui_from_profile(profile);
								thread::sleep(Duration::from_millis(150));
							};
						}
					}

					thread::sleep(Duration::from_millis(50));
				}
			}
		});

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
						Message::Refresh => {
							let profile = app.create_profile_from_gui();

							app.update(profile.effect);
							app::awake();

							manager.set_effect(profile.effect, profile.direction, &profile.rgb_array, profile.speed, profile.brightness);
						}
						Message::CustomEffect { effect } => {
							app.color_tiles.deactivate();
							effect.play(&mut manager);
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

#[derive(Clone)]
pub struct SharedVec<T> {
	inner: Arc<Mutex<Vec<T>>>,
}

impl<T> SharedVec<T> {
	pub fn new() -> Self {
		Self {
			inner: Arc::new(Mutex::new(Vec::new())),
		}
	}

	pub fn push(&self, value: T) {
		self.inner.lock().unwrap().push(value);
	}

	pub fn remove(&self, index: usize) -> T {
		self.inner.lock().unwrap().remove(index)
	}

	pub fn len(&self) -> usize {
		self.inner.lock().unwrap().len()
	}
}
