use std::{fs, path::PathBuf};

use chrono::{serde::ts_seconds, Utc};
use serde::{Deserialize, Serialize};

use crate::{profile::Profile, util::StorageTrait};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Settings {
    pub profiles: Vec<Profile>,
    pub updates: Updates,
}

impl Settings {
    pub fn load_or_default(path: &PathBuf) -> Self {
        let mut persist: Self = Self::default();

        if let Ok(string) = fs::read_to_string(path) {
            persist = serde_json::from_str(&string).unwrap_or_default();
        }

        persist
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Updates {
    pub check_for_updates: bool,
    pub skip_version: bool,
    #[serde(with = "ts_seconds")]
    pub last_checked: chrono::DateTime<Utc>,
    pub version_name: Option<String>,
}

impl Default for Updates {
    fn default() -> Self {
        Self {
            check_for_updates: true,
            last_checked: Default::default(),
            version_name: Default::default(),
            skip_version: Default::default(),
        }
    }
}

impl<'a> StorageTrait<'a> for Settings {}
