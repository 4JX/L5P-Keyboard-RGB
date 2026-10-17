use std::time::Instant;

use crate::{Key, RawCode};

/// A keyboard event and the time it was received.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Event {
    /// Time at which the event was received.
    pub time: Instant,
    pub kind: EventKind,
    /// The original operating-system key code.
    ///
    /// Reset events are not associated with a particular key and carry
    /// `None`.
    pub raw_code: Option<RawCode>,
}

/// A change reported by a keyboard.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventKind {
    KeyDown(Key),
    KeyUp(Key),
    /// Clear the pressed-key list, normally because a keyboard was removed.
    Reset,
}
