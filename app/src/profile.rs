use std::{convert::TryInto, path::PathBuf};

use crate::{
	enums::{Direction, Effects},
	storage_trait::StorageTrait,
};

use error_stack::{Result, ResultExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Serialize, Deserialize, Default, Debug)]
pub struct KeyboardZone {
	rgb: [u8; 3],
	enabled: bool,
}

type Zones = [KeyboardZone; 4];

#[derive(Serialize, Deserialize, Clone, Copy, Default, Debug)]
pub struct Profile {
	pub rgb_zones: Zones,
	pub effect: Effects,
	pub direction: Direction,
	pub speed: u8,
	pub brightness: u8,
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

	pub fn rgb_array(&self) -> [u8; 12] {
		self.rgb_zones.map(|zone| if zone.enabled { zone.rgb } else { [0; 3] }).concat().try_into().unwrap()
	}
}

pub fn arr_to_zones(arr: [u8; 12]) -> Zones {
	[
		KeyboardZone {
			rgb: arr[0..3].try_into().unwrap(),
			enabled: true,
		},
		KeyboardZone {
			rgb: arr[3..6].try_into().unwrap(),
			enabled: true,
		},
		KeyboardZone {
			rgb: arr[6..9].try_into().unwrap(),
			enabled: true,
		},
		KeyboardZone {
			rgb: arr[9..12].try_into().unwrap(),
			enabled: true,
		},
	]
}

impl<'a> StorageTrait<'a> for Profile {}
