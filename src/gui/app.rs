use super::options::OptionsTile;
use super::utils::screen_center;
use super::{color_tiles, options, side_tile};
use super::{color_tiles::ColorTiles, enums::GuiMessage};
use crate::keyboard_manager::{KeyboardManager, StopSignals};
use crate::profile::Profile;
use crate::{
	custom_effect::CustomEffect,
	enums::{Direction, Effects, Message},
};
use crate::{gui::dialog as appdialog, profile::ProfilesData};
use crate::{gui::menu_bar, profile::Profiles};
use clap::crate_name;
use device_query::{DeviceQuery, DeviceState, Keycode};
use fltk::browser::HoldBrowser;
use fltk::dialog;
use fltk::enums::FrameType;
use fltk::{app, enums::Font, prelude::*, window::Window};
use flume::Sender;
use single_instance::SingleInstance;
use std::time::Duration;
use std::{panic, path, thread};
use std::{path::PathBuf, str::FromStr};
use tray_item::{IconSource, TrayItem};

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
	profiles: Profiles,
}

impl App {
	pub fn start_ui(show_window: bool) {
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

		let (window_sender, window_receiver) = flume::unbounded::<GuiMessage>();

		let mut win = Self::create_window(manager);
		win.set_callback(|win| win.hide());

		if show_window {
			win.show()
		};

		//Create the tray icon
		#[cfg(target_os = "linux")]
		let tray_icon = load_icon_data(include_bytes!("../../res/trayIcon.ico"));

		#[cfg(target_os = "linux")]
		let mut tray = TrayItem::new("Keyboard RGB", tray_icon).unwrap();

		#[cfg(target_os = "windows")]
		let mut tray = TrayItem::new("Keyboard RGB", IconSource::Resource("trayIcon")).unwrap();

		tray.add_menu_item("Show", move || window_sender.send(GuiMessage::ShowWindow).unwrap()).unwrap();

		tray.add_menu_item("Quit", || {
			std::process::exit(0);
		})
		.unwrap();

		loop {
			app::wait_for(0.2).unwrap();
			if let Ok(msg) = window_receiver.try_recv() {
				match msg {
					GuiMessage::ShowWindow => win.show(),
					GuiMessage::HideWindow => win.hide(),
				};
			};
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
			if let Ok(profile) = Profile::load_profile(PathBuf::from_str(&filename).unwrap()) {
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

		Profile {
			rgb_array,
			effect,
			direction,
			speed,
			brightness,
			ui_toggle_button_state,
		}
	}

	pub fn save_profile(&mut self) {
		let profile = self.create_profile_from_gui();

		let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseSaveFile);
		dlg.set_option(dialog::FileDialogOptions::SaveAsConfirm);
		dlg.show();

		let filename = dlg.filename().to_string_lossy().to_string();

		if !filename.is_empty() {
			profile.save_profile(filename.as_str()).unwrap();
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
			profiles: Profiles::from_disk(),
		};

		update_preset_browser(&mut side_tile.preset_browser, &app.profiles);

		menu_bar::AppMenuBar::new(&app);

		side_tile.add_preset_button.set_callback({
			let mut app = app.clone();
			let mut preset_browser = side_tile.preset_browser.clone();
			move |_button| {
				let profile = app.create_profile_from_gui();
				app.profiles.push(profile);
				update_preset_browser(&mut preset_browser, &app.profiles);
				ProfilesData::new(&app.profiles).save_profiles().unwrap();
			}
		});

		side_tile.remove_preset_button.set_callback({
			let mut app = app.clone();
			let mut preset_browser = side_tile.preset_browser.clone();
			move |_button| {
				if preset_browser.value() > 0 && !app.profiles.is_empty() {
					app.profiles.remove(preset_browser.value() as usize - 1);
					update_preset_browser(&mut preset_browser, &app.profiles);
					ProfilesData::new(&app.profiles).save_profiles().unwrap();
				}
			}
		});

		side_tile.preset_browser.set_callback({
			let mut app = app.clone();
			let preset_browser = side_tile.preset_browser.clone();
			move |_browser| {
				let profiles = app.profiles.inner.lock().clone();
				if let Some(profile) = profiles.get(preset_browser.value() as usize - 1) {
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
					if !app.profiles.is_empty() {
						let keys: Vec<Keycode> = device_state.get_keys();

						if keys.contains(&Keycode::Meta) && keys.contains(&Keycode::RAlt) {
							if app.profiles.len() > 1 {
								if app.profiles.len() == preset_browser.value() as usize {
									preset_browser.select(1);
								} else {
									preset_browser.select(preset_browser.value() + 1);
								}
							} else {
								preset_browser.select(1);
							}

							let profiles = app.profiles.inner.lock().clone();

							if let Some(profile) = profiles.get(preset_browser.value() as usize - 1) {
								app.update_gui_from_profile(&profile.clone());
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

#[cfg(target_os = "linux")]
pub fn load_icon_data(image_data: &[u8]) -> IconSource {
	let image = image::load_from_memory(image_data).unwrap();
	let image_buffer = image.to_rgba8();
	let pixels = image_buffer.as_raw().clone();

	IconSource::Data {
		data: pixels,
		width: image.width() as i32,
		height: image.height() as i32,
	}
}

fn update_preset_browser(preset_browser: &mut HoldBrowser, profiles: &Profiles) {
	preset_browser.clear();

	for i in 0..profiles.len() {
		preset_browser.add(format!("Preset {}", i).as_str());
	}
}
