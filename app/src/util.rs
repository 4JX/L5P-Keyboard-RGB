use clap::crate_name;
use eframe::egui::Ui;
use error_stack::{Result, ResultExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use single_instance::SingleInstance;
use std::{fs::File, io::Write, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Failed to load file")]
pub struct LoadFileError;

#[derive(Debug, Error)]
#[error("Failed to save file")]
pub struct SaveFileError;

pub(super) trait StorageTrait<'a>
where
    Self: DeserializeOwned + Serialize + Sized,
    for<'de> Self: Deserialize<'de> + 'a,
{
    fn load(path: &Path) -> Result<Self, LoadFileError> {
        let file = std::fs::File::open(path).change_context(LoadFileError)?;

        let reader = std::io::BufReader::new(file);

        serde_json::de::from_reader(reader).change_context(LoadFileError)
    }

    fn save(&self, path: &Path) -> Result<(), SaveFileError> {
        let mut file = File::create(path).change_context(SaveFileError)?;

        let stringified_json = serde_json::to_string(&self).change_context(SaveFileError)?;

        file.write_all(stringified_json.as_bytes()).change_context(SaveFileError)?;

        Ok(())
    }
}

pub fn is_unique_instance() -> bool {
    SingleInstance::new(crate_name!()).unwrap().is_single()
}

pub fn clickable_link(ui: &mut Ui, url: &str) {
    if ui.link(url).clicked() {
        open::that(url).unwrap();
    }
}
