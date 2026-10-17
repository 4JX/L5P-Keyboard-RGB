//! Physical keys whose Linux evdev code matches their non-extended PC set-1
//! scan code.
//!
//! These codes are the same on Linux and Windows, so they are kept in one
//! table. Each operating system handles the remaining keys in its own file.

use crate::Key;

pub(crate) fn from_set1(code: u16) -> Option<Key> {
    Some(match code {
        1 => Key::Escape,
        2 => Key::Digit1,
        3 => Key::Digit2,
        4 => Key::Digit3,
        5 => Key::Digit4,
        6 => Key::Digit5,
        7 => Key::Digit6,
        8 => Key::Digit7,
        9 => Key::Digit8,
        10 => Key::Digit9,
        11 => Key::Digit0,
        12 => Key::Minus,
        13 => Key::Equal,
        14 => Key::Backspace,
        15 => Key::Tab,
        16 => Key::Q,
        17 => Key::W,
        18 => Key::E,
        19 => Key::R,
        20 => Key::T,
        21 => Key::Y,
        22 => Key::U,
        23 => Key::I,
        24 => Key::O,
        25 => Key::P,
        26 => Key::BracketLeft,
        27 => Key::BracketRight,
        28 => Key::Enter,
        29 => Key::ControlLeft,
        30 => Key::A,
        31 => Key::S,
        32 => Key::D,
        33 => Key::F,
        34 => Key::G,
        35 => Key::H,
        36 => Key::J,
        37 => Key::K,
        38 => Key::L,
        39 => Key::Semicolon,
        40 => Key::Quote,
        41 => Key::Backtick,
        42 => Key::ShiftLeft,
        43 => Key::Backslash,
        44 => Key::Z,
        45 => Key::X,
        46 => Key::C,
        47 => Key::V,
        48 => Key::B,
        49 => Key::N,
        50 => Key::M,
        51 => Key::Comma,
        52 => Key::Period,
        53 => Key::Slash,
        54 => Key::ShiftRight,
        55 => Key::NumpadMultiply,
        56 => Key::AltLeft,
        57 => Key::Space,
        58 => Key::CapsLock,
        59 => Key::F1,
        60 => Key::F2,
        61 => Key::F3,
        62 => Key::F4,
        63 => Key::F5,
        64 => Key::F6,
        65 => Key::F7,
        66 => Key::F8,
        67 => Key::F9,
        68 => Key::F10,
        69 => Key::NumLock,
        70 => Key::ScrollLock,
        71 => Key::Numpad7,
        72 => Key::Numpad8,
        73 => Key::Numpad9,
        74 => Key::NumpadSubtract,
        75 => Key::Numpad4,
        76 => Key::Numpad5,
        77 => Key::Numpad6,
        78 => Key::NumpadAdd,
        79 => Key::Numpad1,
        80 => Key::Numpad2,
        81 => Key::Numpad3,
        82 => Key::Numpad0,
        83 => Key::NumpadDecimal,
        86 => Key::IntlBackslash,
        87 => Key::F11,
        88 => Key::F12,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_the_shared_pc_set1_range() {
        assert_eq!(from_set1(30), Some(Key::A));
        assert_eq!(from_set1(54), Some(Key::ShiftRight));
        assert_eq!(from_set1(88), Some(Key::F12));
        assert_eq!(from_set1(0), None);
    }
}
