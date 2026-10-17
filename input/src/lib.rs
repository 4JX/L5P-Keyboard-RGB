//! Reads global keyboard input without changing or blocking it.
//!
//! The operating-system code sends events to a common thread. That thread
//! keeps track of pressed keys and sends the events to subscribers. Cloned
//! [`Input`] handles share the same thread and pressed-key list.

#![forbid(unsafe_op_in_unsafe_fn)]

mod error;
mod event;
mod key;
mod keycodes;
mod platform;

use std::collections::HashSet;
use std::sync::mpsc::{self, Receiver, RecvError, RecvTimeoutError, TryRecvError};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub use error::Error;
pub use event::{Event, EventKind};
pub use key::{Key, RawCode};

/// A shared keyboard listener.
///
/// Creating an input source starts one operating-system thread. Clones share
/// that thread and its pressed-key list. Dropping the final clone stops it.
#[derive(Clone)]
pub struct Input {
    inner: Arc<Inner>,
}

impl Input {
    /// Start monitoring all keyboards visible to this process.
    pub fn new() -> Result<Self, Error> {
        let state = Arc::new(State::default());
        // The platform thread reads keyboard events. The dispatcher below
        // keeps the pressed-key list and passes events to subscribers.
        let (sender, events) = mpsc::channel();
        let platform = platform::start(sender)?;
        let worker_state = Arc::clone(&state);
        let dispatcher = thread::Builder::new()
            .name("legion-input-dispatch".into())
            .spawn(move || dispatch(events, &worker_state))
            .map_err(|error| Error::Platform(format!("could not spawn input dispatcher: {error}")))?;

        Ok(Self {
            inner: Arc::new(Inner {
                _runtime: Runtime {
                    platform: Some(platform),
                    dispatcher: Some(dispatcher),
                },
                state,
            }),
        })
    }

    /// Subscribe to future input events.
    pub fn subscribe(&self) -> Subscription {
        let (sender, events) = mpsc::channel();
        self.inner.state.subscribers.lock().unwrap().push(sender);
        Subscription { events }
    }

    /// Whether `key` is currently held.
    pub fn is_pressed(&self, key: Key) -> bool {
        self.inner.state.held.read().unwrap().contains(&key)
    }

    /// Whether at least one key is currently held.
    pub fn any_pressed(&self) -> bool {
        !self.inner.state.held.read().unwrap().is_empty()
    }

    /// A copy of the keys which are currently held.
    pub fn pressed_keys(&self) -> HashSet<Key> {
        self.inner.state.held.read().unwrap().clone()
    }

    /// Whether every key in `keys` is currently held.
    pub fn chord_pressed(&self, keys: impl IntoIterator<Item = Key>) -> bool {
        let held = self.inner.state.held.read().unwrap();
        keys.into_iter().all(|key| held.contains(&key))
    }
}

/// A stream of future events from an [`Input`] source.
#[derive(Debug)]
pub struct Subscription {
    events: Receiver<Event>,
}

impl Subscription {
    /// Wait for the next input event.
    pub fn recv(&self) -> Result<Event, RecvError> {
        self.events.recv()
    }

    /// Try to receive an input event without blocking.
    pub fn try_recv(&self) -> Result<Event, TryRecvError> {
        self.events.try_recv()
    }

    /// Wait up to `timeout` for the next input event.
    pub fn recv_timeout(&self, timeout: Duration) -> Result<Event, RecvTimeoutError> {
        self.events.recv_timeout(timeout)
    }
}

struct Inner {
    // Stop the threads before dropping the data they use.
    _runtime: Runtime,
    state: Arc<State>,
}

struct Runtime {
    platform: Option<platform::ShutdownGuard>,
    dispatcher: Option<JoinHandle<()>>,
}

impl Drop for Runtime {
    fn drop(&mut self) {
        // Stop the operating-system thread first. This closes the channel and
        // allows the second thread to finish too.
        drop(self.platform.take());
        if let Some(dispatcher) = self.dispatcher.take() {
            let _ = dispatcher.join();
        }
    }
}

#[derive(Default)]
struct State {
    // Every Input handle reads this same set.
    held: RwLock<HashSet<Key>>,
    subscribers: Mutex<Vec<mpsc::Sender<Event>>>,
}

impl State {
    fn publish(&self, event: Event) {
        // Ignore key repeat and any duplicate press or release event.
        let changed = {
            let mut held = self.held.write().unwrap();
            match event.kind {
                EventKind::KeyDown(key) => held.insert(key),
                EventKind::KeyUp(key) => held.remove(&key),
                EventKind::Reset => {
                    held.clear();
                    true
                }
            }
        };

        if changed {
            self.subscribers.lock().unwrap().retain(|subscriber| subscriber.send(event).is_ok());
        }
    }
}

fn dispatch(events: Receiver<Event>, state: &State) {
    while let Ok(event) = events.recv() {
        state.publish(event);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    fn event(kind: EventKind) -> Event {
        Event {
            time: Instant::now(),
            kind,
            raw_code: Some(RawCode::Linux { evdev: 30 }),
        }
    }

    #[test]
    fn dispatch_tracks_held_keys_and_suppresses_duplicate_transitions() {
        let state = State::default();
        let (sender, events) = mpsc::channel();
        state.subscribers.lock().unwrap().push(sender);

        let down = event(EventKind::KeyDown(Key::A));
        let up = event(EventKind::KeyUp(Key::A));
        let duplicate_down = down;

        for input in [down, duplicate_down, up] {
            state.publish(input);
        }

        assert_eq!(events.try_iter().collect::<Vec<_>>(), vec![down, up]);
        assert!(state.held.read().unwrap().is_empty());
    }

    #[test]
    fn reset_clears_held_keys_and_is_published() {
        let state = State::default();
        let (sender, events) = mpsc::channel();
        state.subscribers.lock().unwrap().push(sender);

        let down = event(EventKind::KeyDown(Key::A));
        let reset = Event {
            time: Instant::now(),
            kind: EventKind::Reset,
            raw_code: None,
        };

        state.publish(down);
        state.publish(reset);

        assert_eq!(events.try_iter().collect::<Vec<_>>(), vec![down, reset]);
        assert!(state.held.read().unwrap().is_empty());
    }
}
