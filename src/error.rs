use serde_json::Error as SerdeError;
use std::io::Error as IoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("IoError: {}", .0)]
	IoError(#[from] IoError),
	#[error("SerdeError: {}",.0)]
	SerdeError(#[from] SerdeError),
}
