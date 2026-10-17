//! Linux evdev discovery and event loop.

mod keycodes;

use std::collections::HashSet;
use std::io::{Read, Write};
use std::os::fd::AsFd;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use evdev::{Device, EventSummary, KeyCode};
use nix::poll::{poll, PollFd, PollFlags};

use crate::{Error, Event, EventKind, RawCode};

const HOTPLUG_INTERVAL: Duration = Duration::from_secs(1);
// A device must have an A key to be treated as a keyboard. This filters out
// devices which only report a few buttons.
const KEYBOARD_PROBE: KeyCode = KeyCode::KEY_A;

#[derive(Debug)]
pub(crate) struct ShutdownGuard {
    wake: UnixStream,
    worker: Option<JoinHandle<()>>,
}

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        // The worker also waits on this socket, so writing to it wakes the
        // worker and lets it stop immediately.
        let _ = self.wake.write_all(&[1]);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

struct KeyboardDevice {
    path: PathBuf,
    device: Device,
}

pub(crate) fn start(sender: Sender<Event>) -> Result<ShutdownGuard, Error> {
    let devices = scan_keyboards()?;
    if devices.is_empty() {
        return Err(Error::NoDevices);
    }

    let (wake, worker_wake) = UnixStream::pair()?;
    let worker = thread::Builder::new()
        .name("legion-input-linux".into())
        .spawn(move || run(devices, worker_wake, sender))
        .map_err(|error| Error::Platform(format!("could not spawn evdev worker: {error}")))?;

    Ok(ShutdownGuard { wake, worker: Some(worker) })
}

fn run(mut devices: Vec<KeyboardDevice>, mut wake: UnixStream, sender: Sender<Event>) {
    let mut last_scan = Instant::now();

    loop {
        // The first entry is the shutdown socket. The rest are in the same
        // order as the devices list.
        let mut poll_fds = Vec::with_capacity(devices.len() + 1);
        poll_fds.push(PollFd::new(wake.as_fd(), PollFlags::POLLIN));
        poll_fds.extend(devices.iter().map(|keyboard| PollFd::new(keyboard.device.as_fd(), PollFlags::POLLIN)));

        match poll(&mut poll_fds, 1_000u16) {
            Ok(_) => {}
            Err(nix::errno::Errno::EINTR) => continue,
            Err(_) => {
                let _ = sender.send(Event {
                    time: Instant::now(),
                    kind: EventKind::Reset,
                    raw_code: None,
                });
                break;
            }
        }

        let shutdown = poll_fds[0].revents().is_some_and(|events| events.contains(PollFlags::POLLIN));
        let readiness = poll_fds[1..].iter().map(|fd| fd.revents().unwrap_or_else(PollFlags::empty)).collect::<Vec<_>>();
        drop(poll_fds);

        if shutdown {
            let mut byte = [0];
            let _ = wake.read(&mut byte);
            break;
        }

        let mut removed_any = false;
        let mut index = 0;
        devices.retain_mut(|keyboard| {
            let ready = readiness[index];
            index += 1;

            if ready.intersects(PollFlags::POLLERR | PollFlags::POLLHUP | PollFlags::POLLNVAL) {
                removed_any = true;
                return false;
            }
            if !ready.contains(PollFlags::POLLIN) {
                return true;
            }

            match keyboard.device.fetch_events() {
                Ok(events) => {
                    for raw_event in events {
                        let EventSummary::Key(_, code, value) = raw_event.destructure() else {
                            continue;
                        };
                        let key = keycodes::from_evdev(code.0);
                        let kind = match value {
                            0 => EventKind::KeyUp(key),
                            1 => EventKind::KeyDown(key),
                            _ => continue,
                        };
                        if sender
                            .send(Event {
                                time: Instant::now(),
                                kind,
                                raw_code: Some(RawCode::Linux { evdev: code.0 }),
                            })
                            .is_err()
                        {
                            return false;
                        }
                    }
                    true
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => true,
                Err(_) => {
                    removed_any = true;
                    false
                }
            }
        });

        if removed_any {
            // If a keyboard disappears while a key is down, its release event
            // will never arrive. Clear all pressed keys instead.
            let _ = sender.send(Event {
                time: Instant::now(),
                kind: EventKind::Reset,
                raw_code: None,
            });
        }

        if removed_any || last_scan.elapsed() >= HOTPLUG_INTERVAL {
            // Check for newly connected keyboards once a second, and also
            // whenever one disappears.
            reconcile_keyboards(&mut devices);
            last_scan = Instant::now();
        }
    }
}

fn reconcile_keyboards(devices: &mut Vec<KeyboardDevice>) {
    let current_paths = devices.iter().map(|keyboard| keyboard.path.clone()).collect::<HashSet<_>>();

    let Ok(entries) = std::fs::read_dir("/dev/input") else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if current_paths.contains(&path) || !is_event_node(&path) {
            continue;
        }
        if let Ok(device) = open_keyboard(path) {
            devices.push(device);
        }
    }
}

fn scan_keyboards() -> Result<Vec<KeyboardDevice>, Error> {
    let entries = std::fs::read_dir("/dev/input")?;
    let mut devices = Vec::new();
    let mut saw_event_node = false;
    let mut saw_permission_denied = false;

    for entry in entries.flatten() {
        let path = entry.path();
        if !is_event_node(&path) {
            continue;
        }
        saw_event_node = true;
        match open_keyboard(path) {
            Ok(device) => devices.push(device),
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                saw_permission_denied = true;
            }
            Err(_) => {}
        }
    }

    if devices.is_empty() && saw_event_node && saw_permission_denied {
        return Err(Error::PermissionDenied);
    }
    Ok(devices)
}

fn open_keyboard(path: PathBuf) -> std::io::Result<KeyboardDevice> {
    let device = Device::open(&path)?;
    let is_keyboard = device.supported_keys().is_some_and(|keys| keys.contains(KEYBOARD_PROBE));
    if !is_keyboard {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "not a keyboard input device"));
    }

    device.set_nonblocking(true)?;
    Ok(KeyboardDevice { path, device })
}

fn is_event_node(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.starts_with("event"))
}
