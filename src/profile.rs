use crate::{enums::Effects, error};
use serde::{Deserialize, Serialize};
use std::{
	fs::{self, File},
	io::{BufWriter, Write},
	path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
pub struct Profile {
	pub rgb_array: [u8; 12],
	pub effect: Effects,
	pub speed: u8,
	pub brightness: u8,
	pub ui_toggle_button_state: [bool; 5],
}

impl Profile {
	pub fn new(rgb_array: [u8; 12], effect: Effects, speed: u8, brightness: u8, ui_toggle_button_state: [bool; 5]) -> Self {
		Self {
			rgb_array: rgb_array.clone(),
			effect,
			speed,
			brightness,
			ui_toggle_button_state,
		}
	}
	pub fn save(&self, profile_name: &str) -> Result<(), error::Error> {
		let profile_path = Self::get_full_path(profile_name.to_string());
		let file = File::create(&profile_path)?;
		let struct_json = serde_json::to_string(self)?;
		println!("{}", struct_json);
		let mut w = BufWriter::new(file);
		w.write_all(struct_json.as_bytes()).unwrap();
		w.flush().unwrap();

		Ok(())
	}

	pub fn from_file(mut path_string: String) -> Result<Self, error::Error> {
		if !path_string.ends_with(".json") {
			path_string = format!("{}{}", path_string, ".json");
		}
		let path = Path::new(&path_string);
		let full_path = fs::canonicalize(path)?;
		let struct_json = fs::read_to_string(&full_path)?;
		let profile: Profile = serde_json::from_str(struct_json.as_str())?;
		Ok(profile)
	}

	fn get_current_dir() -> PathBuf {
		let current_dir = std::env::current_dir().unwrap();
		current_dir
	}

	fn get_full_path(mut profile_name: String) -> PathBuf {
		let config_dir = Self::get_current_dir();
		if !profile_name.ends_with(".json") {
			profile_name = format!("{}{}", profile_name, ".json");
		}
		Path::new(&config_dir).join(format! {"{}{}", profile_name, ".json"})
	}
}
