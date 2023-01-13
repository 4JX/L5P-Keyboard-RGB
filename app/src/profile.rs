use std::{convert::TryInto, path::PathBuf};

use crate::{
    enums::{Direction, Effects},
    util::StorageTrait,
};

use error_stack::{Result, ResultExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
pub struct KeyboardZone {
    pub rgb: [u8; 3],
    pub enabled: bool,
}

impl Default for KeyboardZone {
    fn default() -> Self {
        Self {
            rgb: Default::default(),
            enabled: true,
        }
    }
}

type Zones = [KeyboardZone; 4];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Profile {
    pub name: String,
    pub rgb_zones: Zones,
    pub effect: Effects,
    pub direction: Direction,
    pub speed: u8,
    pub brightness: u8,
}

// Primarily differentiated by name but the rest is left for fuckups
impl PartialEq for Profile {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.rgb_zones == other.rgb_zones
            && self.effect == other.effect
            && self.direction == other.direction
            && self.speed == other.speed
            && self.brightness == other.brightness
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: "Profile".to_string(),
            rgb_zones: Default::default(),
            effect: Default::default(),
            direction: Default::default(),
            speed: 1,
            brightness: 1,
        }
    }
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
