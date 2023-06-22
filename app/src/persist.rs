use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{profile::Profile, util::StorageTrait};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Settings {
    pub profiles: Vec<Profile>,
    pub ui_state: Profile,
}

impl Settings {
    /// Load the settings from the specified path or generate default ones if an error occurs
    pub fn load_or_default(path: &Path) -> Self {
        let mut persist: Self = Self::default();

        if let Ok(string) = fs::read_to_string(path) {
            persist = serde_json::from_str(&string).unwrap_or_default();
        }

        persist
    }
}

impl<'a> StorageTrait<'a> for Settings {}
