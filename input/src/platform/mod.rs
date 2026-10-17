use std::sync::mpsc::Sender;

use crate::{Error, Event};

// Select the right implementation here so the rest of the crate can call the
// same start function on Linux and Windows.
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as implementation;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as implementation;

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
compile_error!("legion-input currently supports only Linux and Windows");

pub(crate) use implementation::ShutdownGuard;

pub(crate) fn start(sender: Sender<Event>) -> Result<ShutdownGuard, Error> {
    implementation::start(sender)
}
