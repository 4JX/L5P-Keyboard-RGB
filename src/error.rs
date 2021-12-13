use hidapi::HidError;
use serde_json::Error as SerdeError;
use std::io::Error as IoError;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
	#[error("IoError: {}", .0)]
	IoError(#[from] IoError),
	#[error("SerdeError: {}",.0)]
	SerdeError(#[from] SerdeError),
	#[error("HidError: {}",.0)]
	HidError(#[from] HidError),
	#[error("Error: Couldn't find device")]
	DeviceNotFound,
}
