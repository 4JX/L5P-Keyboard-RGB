use serde_json::Error as SerdeError;
use std::io::Error as IoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error(transparent)]
	IoError(#[from] IoError),
	#[error(transparent)]
	SerdeError(#[from] SerdeError),
}
