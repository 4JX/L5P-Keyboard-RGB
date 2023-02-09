use std::{fs, path::Path};

use chrono::{serde::ts_seconds, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{profile::Profile, util::StorageTrait};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Settings {
    pub profiles: Vec<Profile>,
    pub updates: Updates,
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

    /// Same as `load_or_default` but it also performs an update check and updates values accordingly
    pub fn load_with_check(path: &Path) -> Self {
        let mut settings = Self::load_or_default(path);

        let version_name = &mut settings.updates.version_name;

        let time_since_last_check = Utc::now() - settings.updates.last_checked;

        if settings.updates.check_for_updates && time_since_last_check > Duration::days(1) {
            let client = reqwest::blocking::Client::builder()
                .user_agent(format!("4JX/L5P-Keyboard-RGB, Ver {}", env!("CARGO_PKG_VERSION")))
                .build()
                .unwrap();

            if let Ok(res) = client.get("https://api.github.com/repos/4JX/L5P-Keyboard-RGB/tags").send() {
                let json: Value = res.json().unwrap();

                if let Some(entry) = json.pointer("/0/name") {
                    let mut name = entry.to_string().replace('\"', "");

                    match version_name {
                        Some(current_name) => {
                            if settings.updates.skip_version && current_name != name.as_mut() {
                                *current_name = name;
                                settings.updates.skip_version = false;
                            }
                        }
                        None => {
                            *version_name = Some(name);
                            settings.updates.skip_version = false;
                        }
                    }
                }
            };

            settings.updates.last_checked = Utc::now();
        }

        if version_name.is_some() {
            let n = version_name.as_ref().unwrap();

            if n.is_empty() || n == concat!("v", env!("CARGO_PKG_VERSION")) {
                *version_name = None;
            }
        }

        settings
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
