use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
	fs::{self, File},
	io::Write,
	path::PathBuf,
};

use crate::error;

pub(super) trait StorageTrait<'a>
where
	Self: DeserializeOwned + Serialize + Default + Sized,
	for<'de> Self: Deserialize<'de> + 'a,
{
	const FILE_NAME: &'static str;

	fn load(folder_path: PathBuf, file_name: Option<String>) -> Result<Self, error::Error> {
		let file_name = file_name.unwrap_or_else(|| Self::FILE_NAME.to_string());

		if !folder_path.exists() {
			fs::create_dir_all(&folder_path)?;
		}

		let path = folder_path.join(file_name);
		match std::fs::File::open(&path) {
			Ok(file) => {
				let reader = std::io::BufReader::new(file);
				Ok(serde_json::de::from_reader(reader)?)
			}
			Err(err) => match err.kind() {
				std::io::ErrorKind::NotFound => {
					let new_value = Self::default();
					let mut file = File::create(path)?;
					file.write_all(serde_json::to_string(&new_value)?.as_bytes())?;
					Ok(new_value)
				}

				_ => Err(err.into()),
			},
		}
	}

	fn save(&self, folder_path: PathBuf, file_name: Option<String>) -> Result<(), error::Error> {
		let file_name = file_name.unwrap_or_else(|| Self::FILE_NAME.to_string());

		if !folder_path.exists() {
			fs::create_dir_all(&folder_path)?;
		}
		let path = folder_path.join(file_name);
		let mut file = File::create(path)?;
		let stringified_json = serde_json::to_string(&self)?;
		file.write_all(stringified_json.as_bytes())?;
		Ok(())
	}
}
