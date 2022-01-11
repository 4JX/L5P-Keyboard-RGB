use crate::{
	enums::{Direction, Effects},
	error,
};
use serde::{Deserialize, Serialize};
use std::{
	fs::{self, File},
	io::{BufWriter, Write},
	path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Profile {
	pub rgb_array: [u8; 12],
	pub effect: Effects,
	pub direction: Direction,
	pub speed: u8,
	pub brightness: u8,
	pub ui_toggle_button_state: [bool; 5],
}

impl Profile {
	pub const fn new(rgb_array: [u8; 12], effect: Effects, direction: Direction, speed: u8, brightness: u8, ui_toggle_button_state: [bool; 5]) -> Self {
		Self {
			rgb_array,
			effect,
			direction,
			speed,
			brightness,
			ui_toggle_button_state,
		}
	}
	pub fn save(&self, profile_name: &str) -> Result<(), error::Error> {
		let profile_path = Self::get_full_path(profile_name.to_string());
		let file = File::create(&profile_path)?;
		let struct_json = serde_json::to_string(self)?;
		let mut w = BufWriter::new(file);
		w.write_all(struct_json.as_bytes()).unwrap();
		w.flush().unwrap();

		Ok(())
	}

	pub fn from_file(mut path_string: String) -> Result<Self, error::Error> {
		if path_string.rsplit('.').next().map(|ext| ext.eq_ignore_ascii_case("json")) != Some(true) {
			path_string = format!("{}{}", path_string, ".json");
		}
		let path = Path::new(&path_string);
		let full_path = fs::canonicalize(path)?;
		let struct_json = fs::read_to_string(&full_path)?;
		let profile: Self = serde_json::from_str(struct_json.as_str())?;
		Ok(profile)
	}

	fn get_current_dir() -> PathBuf {
		std::env::current_dir().unwrap()
	}

	fn get_full_path(mut profile_name: String) -> PathBuf {
		let config_dir = Self::get_current_dir();
		if profile_name.rsplit('.').next().map(|ext| ext.eq_ignore_ascii_case("json")) != Some(true) {
			profile_name = format!("{}{}", profile_name, ".json");
		}
		Path::new(&config_dir).join(profile_name)
	}
}
