use super::{
	builder::EFFECTS_LIST,
	color_tiles::{ColorTiles, ColorTilesState},
};
use crate::{
	enums::{Effects, Message},
	gui::dialog::alert,
};
use fltk::{
	browser::HoldBrowser,
	dialog,
	menu::Choice,
	prelude::{BrowserExt, MenuExt},
	text,
};
use serde::{Deserialize, Serialize};
use serde_json::Result;

use std::{
	path,
	str::FromStr,
	sync::{
		atomic::{AtomicBool, Ordering},
		mpsc::Sender,
		Arc,
	},
};

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
	pub speed_choice: Choice,
	pub brightness_choice: Choice,
	pub tx: Sender<Message>,
	pub stop_signal: Arc<AtomicBool>,
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

		if !filename.is_empty() {
			fn parse_profile(profile_text: &str) -> Result<Profile> {
				serde_json::from_str(profile_text)
			}

			if path::Path::new(&filename).exists() {
				self.buf.load_file(&filename).unwrap();
				match parse_profile(&self.buf.text()) {
					Ok(profile) => {
						self.color_tiles.set_state(&profile.color_tiles_state);
						self.effect_browser.select(EFFECTS_LIST.iter().position(|&val| val == profile.effect.to_string()).unwrap() as i32 + 1);
						self.speed_choice.set_value(profile.speed);
						self.brightness_choice.set_value(profile.brightness);
						self.stop_signal.store(true, Ordering::SeqCst);
						self.tx.send(Message::UpdateEffect { effect: profile.effect }).unwrap();
					}
					Err(_) => {
						alert(
							800,
							200,
							"There was an error loading the profile.\nPlease make sure its a valid profile file and that it is compatible with this version of the program.",
						);
						self.stop_signal.store(true, Ordering::SeqCst);
						self.tx.send(Message::Refresh).unwrap();
					}
				}
			} else {
				alert(800, 200, "File does not exist!");
			}
		} else {
			self.stop_signal.store(true, Ordering::SeqCst);
			self.tx.send(Message::Refresh).unwrap();
		}
	}
	pub fn save_profile(&mut self) {
		let profile = Profile {
			color_tiles_state: self.color_tiles.get_state(),
			effect: Effects::from_str(EFFECTS_LIST[self.effect_browser.value() as usize - 1]).unwrap(),
			speed: self.speed_choice.value(),
			brightness: self.brightness_choice.value(),
		};

		self.buf.set_text(serde_json::to_string(&profile).unwrap().as_str());

		let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseSaveFile);
		dlg.set_option(dialog::FileDialogOptions::SaveAsConfirm);
		dlg.show();
		let mut filename = dlg.filename().to_string_lossy().to_string();
		if !filename.is_empty() {
			if !filename.ends_with(".json") {
				filename = format!("{}{}", &filename, ".json");
			}
			self.buf.save_file(filename).unwrap_or_else(|_| alert(800, 200, "Please specify a file name to use."));
		}

		self.stop_signal.store(true, Ordering::SeqCst);
		self.tx.send(Message::Refresh).unwrap();
	}
}
