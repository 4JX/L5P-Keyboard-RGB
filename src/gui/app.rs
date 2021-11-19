use super::color_tiles::{ColorTiles, ColorTilesState};
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
	pub fn load_profile(&mut self) {
		let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseFile);
		dlg.set_option(dialog::FileDialogOptions::NoOptions);
		dlg.set_filter("*.json");
		dlg.show();
		let filename = dlg.filename().to_string_lossy().to_string();
		if !filename.is_empty() {
			fn parse_profile(profile_text: &str) -> Result<Profile> {
				serde_json::from_str(profile_text)
			}

			if path::Path::new(&filename).exists() {
				self.buf.load_file(&filename).unwrap();
			} else {
				alert(800, 200, "File does not exist!");
			}

			match parse_profile(&self.buf.text()) {
				Ok(profile) => {
					self.color_tiles.set_state(&profile.color_tiles_state);
					self.effect_browser.select(0);
					self.speed_choice.set_value(profile.speed);
					self.brightness_choice.set_value(profile.brightness);
					self.stop_signal.store(true, Ordering::SeqCst);
					self.tx.send(Message::UpdateEffect { effect: profile.effect }).unwrap();
				}
				Err(_) => alert(800, 200, "There was an error loading the profile. Please make sure its a valid profile file."),
			}
		}
	}
	pub fn save_profile(&mut self) {
		let profile = Profile {
			color_tiles_state: ColorTilesState {
				rgb_values: self.color_tiles.get_zone_values(),
				buttons_toggle_state: self.color_tiles.get_button_state(),
			},
			effect: Effects::Static,
			speed: self.speed_choice.value(),
			brightness: self.brightness_choice.value(),
		};

		self.buf.set_text(serde_json::to_string(&profile).unwrap().as_str());

		let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseSaveFile);
		dlg.set_option(dialog::FileDialogOptions::SaveAsConfirm);
		dlg.show();
		let filename = dlg.filename().to_string_lossy().to_string();
		if !filename.is_empty() {
			self.buf
				.save_file(format!("{}{}", &filename, ".json"))
				.unwrap_or_else(|_| alert(800, 200, "Please specify a file name to use."));
		}
	}
}
