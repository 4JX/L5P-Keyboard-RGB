use hidapi::HidError;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
	#[error("HidError: {}",.0)]
	HidError(#[from] HidError),
	#[error("Error: Couldn't find device")]
	DeviceNotFound,
}
