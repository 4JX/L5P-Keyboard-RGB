use std::{convert::TryInto, path::Path};

use crate::{
    enums::{Brightness, Direction, Effects},
    util::StorageTrait,
};

use error_stack::{Result, ResultExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Profile {
    pub name: Option<String>,
    pub rgb_zones: Zones,
    pub effect: Effects,
    pub direction: Direction,
    pub speed: u8,
    pub brightness: Brightness,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: None,
            rgb_zones: Zones::default(),
            effect: Effects::default(),
            direction: Direction::default(),
            speed: 1,
            brightness: Brightness::default(),
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
    pub fn load_profile(path: &Path) -> Result<Self, LoadProfileError> {
        Self::load(path).change_context(LoadProfileError)
    }

    pub fn save_profile(&mut self, path: &Path) -> Result<(), SaveProfileError> {
        if self.name.is_none() {
            self.name = Some("Untitled".to_string());
        }
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
