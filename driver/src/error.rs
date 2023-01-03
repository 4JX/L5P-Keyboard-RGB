use hidapi::HidError;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
	#[error("HidError: {}", .0)]
	HidError(#[from] HidError),
	#[error("Error: Couldn't find device")]
	DeviceNotFound,
	#[error("Error: {}", .0)]
	RangeError(#[from] RangeError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
#[error("RangeError: A value specified was not within the expected range")]
pub struct RangeError {
	pub kind: RangeErrorKind,
}

#[derive(Debug)]
pub enum RangeErrorKind {
	Zone,
	Speed,
	Brightness,
}
