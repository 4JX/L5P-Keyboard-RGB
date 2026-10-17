//! Windows set-1 scan-code mapping.

use crate::{keycodes::from_set1, Key, RawCode};

pub(super) fn from_scan_code(scan_code: u32, virtual_key: u32, extended: bool) -> Key {
    let raw = RawCode::Windows { scan_code, virtual_key, extended };

    // Arrow keys and numpad keys can share a scan code. The extended bit tells
    // them apart, and also identifies several right-hand and special keys.
    if extended {
        return match scan_code {
            0x1C => Key::NumpadEnter,
            0x1D => Key::ControlRight,
            0x35 => Key::NumpadDivide,
            0x37 => Key::PrintScreen,
            0x38 => Key::AltRight,
            0x47 => Key::Home,
            0x48 => Key::ArrowUp,
            0x49 => Key::PageUp,
            0x4B => Key::ArrowLeft,
            0x4D => Key::ArrowRight,
            0x4F => Key::End,
            0x50 => Key::ArrowDown,
            0x51 => Key::PageDown,
            0x52 => Key::Insert,
            0x53 => Key::Delete,
            0x5B => Key::MetaLeft,
            0x5C => Key::MetaRight,
            0x5D => Key::Menu,
            _ => Key::Unknown(raw),
        };
    }

    u16::try_from(scan_code).ok().and_then(from_set1).unwrap_or(Key::Unknown(raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinguishes_extended_keys_from_numpad_keys() {
        assert_eq!(from_scan_code(0x48, 0x68, false), Key::Numpad8);
        assert_eq!(from_scan_code(0x48, 0x26, true), Key::ArrowUp);
        assert_eq!(from_scan_code(0x1C, 0x0D, false), Key::Enter);
        assert_eq!(from_scan_code(0x1C, 0x0D, true), Key::NumpadEnter);
    }

    #[test]
    fn preserves_unknown_extended_codes() {
        assert_eq!(
            from_scan_code(0x7F, 0xFF, true),
            Key::Unknown(RawCode::Windows {
                scan_code: 0x7F,
                virtual_key: 0xFF,
                extended: true,
            })
        );
    }
}
