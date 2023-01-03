use std::path::PathBuf;

use crate::{
	enums::{Direction, Effects},
	storage_trait::StorageTrait,
};

use error_stack::{Result, ResultExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone, Copy, Default, Debug)]
pub struct Profile {
	pub rgb_array: [u8; 12],
	pub effect: Effects,
	pub direction: Direction,
	pub speed: u8,
	pub brightness: u8,
	pub ui_toggle_button_state: [bool; 5],
}

#[derive(Debug, Error)]
#[error("Could not load profile")]
pub struct LoadProfileError;

#[derive(Debug, Error)]
#[error("Could not save profile")]
pub struct SaveProfileError;

impl Profile {
	pub fn load_profile(path: PathBuf) -> Result<Self, LoadProfileError> {
		Self::load(path).change_context(LoadProfileError)
	}

	pub fn save_profile(&self, path: PathBuf) -> Result<(), SaveProfileError> {
		self.save(path).change_context(SaveProfileError)
	}
}

impl<'a> StorageTrait<'a> for Profile {}
