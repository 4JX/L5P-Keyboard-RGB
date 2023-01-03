use clap::crate_name;
use error_stack::{IntoReport, Result, ResultExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use single_instance::SingleInstance;
use std::{fs::File, io::Write, path::PathBuf};
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
	fn load(path: PathBuf) -> Result<Self, LoadFileError> {
		let file = std::fs::File::open(&path).into_report().change_context(LoadFileError)?;

		let reader = std::io::BufReader::new(file);

		Ok(serde_json::de::from_reader(reader).into_report().change_context(LoadFileError)?)
	}

	fn save(&self, path: PathBuf) -> Result<(), SaveFileError> {
		let mut file = File::create(path).into_report().change_context(SaveFileError)?;

		let stringified_json = serde_json::to_string(&self).into_report().change_context(SaveFileError)?;

		file.write_all(stringified_json.as_bytes()).into_report().change_context(SaveFileError)?;

		Ok(())
	}
}

pub fn is_unique_instance() -> bool {
	SingleInstance::new(crate_name!()).unwrap().is_single()
}
