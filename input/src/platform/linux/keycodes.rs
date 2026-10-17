//! evdev key-code mapping.

use crate::{keycodes::from_set1, Key, RawCode};

pub(super) fn from_evdev(code: u16) -> Key {
    match from_set1(code) {
        Some(key) => key,
        None => match code {
            96 => Key::NumpadEnter,
            97 => Key::ControlRight,
            98 => Key::NumpadDivide,
            99 => Key::PrintScreen,
            100 => Key::AltRight,
            102 => Key::Home,
            103 => Key::ArrowUp,
            104 => Key::PageUp,
            105 => Key::ArrowLeft,
            106 => Key::ArrowRight,
            107 => Key::End,
            108 => Key::ArrowDown,
            109 => Key::PageDown,
            110 => Key::Insert,
            111 => Key::Delete,
            119 => Key::Pause,
            125 => Key::MetaLeft,
            126 => Key::MetaRight,
            127 => Key::Menu,
            other => Key::Unknown(RawCode::Linux { evdev: other }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_left_and_right_modifiers_distinct() {
        assert_eq!(from_evdev(42), Key::ShiftLeft);
        assert_eq!(from_evdev(54), Key::ShiftRight);
        assert_eq!(from_evdev(29), Key::ControlLeft);
        assert_eq!(from_evdev(97), Key::ControlRight);
    }

    #[test]
    fn preserves_unknown_codes() {
        assert_eq!(from_evdev(999), Key::Unknown(RawCode::Linux { evdev: 999 }));
    }
}
