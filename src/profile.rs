use crate::{
	enums::{Direction, Effects},
	error,
	storage_trait::StorageTrait,
};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Default)]
pub struct Profiles {
	pub inner: Arc<Mutex<Vec<Profile>>>,
}

#[allow(dead_code)]
impl Profiles {
	pub fn new(profiles_data: ProfilesData) -> Self {
		let inner = Arc::new(Mutex::new(profiles_data.inner));
		Self { inner }
	}

	pub fn len(&self) -> usize {
		self.inner.lock().len()
	}

	pub fn push(&mut self, item: Profile) {
		self.inner.lock().push(item);
	}

	pub fn remove(&mut self, pos: usize) -> Profile {
		self.inner.lock().remove(pos)
	}

	pub fn is_empty(&self) -> bool {
		self.inner.lock().is_empty()
	}

	pub fn from_disk() -> Self {
		let data = ProfilesData::load_profiles().unwrap_or_default();
		Self::new(data)
	}
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ProfilesData {
	pub inner: Vec<Profile>,
}

#[allow(dead_code)]
impl ProfilesData {
	pub fn new(profiles: &Profiles) -> Self {
		let inner = profiles.inner.lock().clone();
		Self { inner }
	}

	pub fn load_profiles() -> Result<Self, error::Error> {
		let current_dir = std::env::current_dir().unwrap();
		Self::load(current_dir, None)
	}

	pub fn save_profiles(&self) -> Result<(), error::Error> {
		let current_dir = std::env::current_dir().unwrap();
		self.save(current_dir, None)
	}
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct Profile {
	pub rgb_array: [u8; 12],
	pub effect: Effects,
	pub direction: Direction,
	pub speed: u8,
	pub brightness: u8,
	pub ui_toggle_button_state: [bool; 5],
}

impl Profile {
	pub fn load_profile(path: PathBuf) -> Result<Self, error::Error> {
		if path.is_file() {
			Self::load(path.parent().unwrap().to_path_buf(), path.file_name().map(|str| str.to_string_lossy().to_string()))
		} else {
			Self::load(path, None)
		}
	}

	pub fn save_profile(&self, filename: &str) -> Result<(), error::Error> {
		let current_dir = std::env::current_dir().unwrap();
		self.save(current_dir, Some(filename.to_string()))
	}
}

impl<'a> StorageTrait<'a> for ProfilesData {
	const FILE_NAME: &'static str = "profiles.json";
}

impl<'a> StorageTrait<'a> for Profile {
	const FILE_NAME: &'static str = "profile.json";
}
