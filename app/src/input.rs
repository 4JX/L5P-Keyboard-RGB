//! Platform-independent input helpers.
//!
//! On Linux, uses evdev (works on both X11 and Wayland).
//! On Windows, device_query is used directly by each consumer.

#[cfg(target_os = "linux")]
use evdev::Device;
#[cfg(target_os = "linux")]
use std::os::fd::{AsRawFd, BorrowedFd};

/// Find a keyboard evdev device by looking for one that supports typical keyboard keys.
#[cfg(target_os = "linux")]
pub fn find_keyboard_device() -> Option<Device> {
    for (_path, device) in evdev::enumerate() {
        if let Some(keys) = device.supported_keys() {
            if keys.contains(evdev::Key::KEY_A) && keys.contains(evdev::Key::KEY_Z) {
                return Some(device);
            }
        }
    }
    None
}

/// Poll an evdev device for pending events with a timeout in milliseconds.
/// Returns `true` if events are available to read.
#[cfg(target_os = "linux")]
pub fn poll_device(device: &Device, timeout_ms: u16) -> bool {
    let fd = unsafe { BorrowedFd::borrow_raw(device.as_raw_fd()) };
    let mut fds = [nix::poll::PollFd::new(fd, nix::poll::PollFlags::POLLIN)];
    nix::poll::poll(&mut fds, timeout_ms).unwrap_or(0) > 0
}
