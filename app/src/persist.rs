use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use crate::profile::Profile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Settings {
    pub profiles: Vec<Profile>,
    // Up to 0.19.5
    #[serde(alias = "ui_state")]
    pub current_profile: Profile,
}

impl Settings {
    /// Load the settings from the configured path or generate default ones if an error occurs
    pub fn load() -> Self {
        let mut persist: Self = Self::default();

        if let Ok(string) = fs::read_to_string(Self::get_location()) {
            persist = serde_json::from_str(&string).unwrap_or_default();
        }

        persist
    }

    /// Save the settings to the configured path
    pub fn save(&mut self) {
        let mut file = File::create(Self::get_location()).unwrap();

        let stringified_json = serde_json::to_string(&self).unwrap();

        file.write_all(stringified_json.as_bytes()).unwrap();
    }

    fn get_location() -> PathBuf {
        let default = PathBuf::from("./settings.json");

        if let Ok(maybe_path) = env::var("LEGION_KEYBOARD_CONFIG") {
            let path = PathBuf::from(maybe_path);
            if path.exists() && path.is_file() {
                path
            } else {
                default
            }
        } else {
            default
        }
    }
}
