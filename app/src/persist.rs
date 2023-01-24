use std::{fs, path::PathBuf};

use chrono::{serde::ts_seconds, Utc};
use serde::{Deserialize, Serialize};

use crate::{profile::Profile, util::StorageTrait};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Persist {
    pub settings: Settings,
    pub data: Data,
}

impl Persist {
    pub fn load_or_default(path: &PathBuf) -> Self {
        let mut persist: Self = Self::default();

        if let Ok(string) = fs::read_to_string(path) {
            persist = serde_json::from_str(&string).unwrap_or_default();
        }

        persist
    }
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub check_for_updates: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { check_for_updates: true }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Data {
    pub profiles: Vec<Profile>,
    pub updates: UpdateData,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct UpdateData {
    #[serde(with = "ts_seconds")]
    pub last_checked: chrono::DateTime<Utc>,
    pub version_name: Option<String>,
    pub skip_version: bool,
}

impl<'a> StorageTrait<'a> for Persist {}
