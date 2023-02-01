use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use rdev::Key;

use crate::profile::Profile;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RippleMove {
    Center,
    Left,
    Right,
    Off,
}

pub(super) struct Ripple;

impl Ripple {
    pub fn play(manager: &mut super::Inner, p: &Profile) {
        // Welcome to the definition of i-don't-know-what-im-doing
        let keys_zone_1: [Key; 23] = [
            Key::Escape,
            Key::F1,
            Key::F2,
            Key::F3,
            Key::F4,
            // Key::Grave,
            Key::Num1,
            Key::Num2,
            Key::Num3,
            Key::Num4,
            Key::Tab,
            Key::KeyQ,
            Key::KeyW,
            Key::KeyE,
            Key::CapsLock,
            Key::KeyA,
            Key::KeyS,
            Key::KeyD,
            Key::ShiftLeft,
            Key::KeyZ,
            Key::KeyX,
            Key::ControlLeft,
            Key::MetaLeft,
            Key::Alt,
        ];

        let keys_zone_2: [Key; 29] = [
            Key::F5,
            Key::F6,
            Key::F7,
            Key::F8,
            Key::F9,
            Key::F10,
            Key::Num5,
            Key::Num6,
            Key::Num7,
            Key::Num8,
            Key::Num9,
            Key::KeyR,
            Key::KeyT,
            Key::KeyY,
            Key::KeyU,
            Key::KeyI,
            Key::KeyF,
            Key::KeyG,
            Key::KeyH,
            Key::KeyJ,
            Key::KeyK,
            Key::KeyC,
            Key::KeyV,
            Key::KeyB,
            Key::KeyN,
            Key::KeyM,
            Key::Comma,
            Key::Space,
            Key::AltGr,
        ];
        let keys_zone_3: [Key; 25] = [
            Key::F11,
            Key::F12,
            Key::Insert,
            Key::Delete,
            Key::Num0,
            Key::Minus,
            Key::Equal,
            Key::Backspace,
            Key::KeyO,
            Key::KeyP,
            Key::LeftBracket,
            Key::RightBracket,
            Key::Return,
            Key::KeyL,
            Key::SemiColon,
            Key::Quote,
            Key::BackSlash,
            Key::Dot,
            Key::Slash,
            Key::ShiftRight,
            Key::ControlRight,
            Key::UpArrow,
            Key::DownArrow,
            Key::LeftArrow,
            Key::RightArrow,
        ];

        let keys_zone_4: [Key; 18] = [
            Key::Home,
            Key::End,
            Key::PageUp,
            Key::PageDown,
            Key::KpDivide,
            Key::KpMultiply,
            Key::KpMinus,
            Key::Kp7,
            Key::Kp8,
            Key::Kp9,
            Key::Kp4,
            Key::Kp5,
            Key::Kp6,
            Key::KpPlus,
            Key::Kp1,
            Key::Kp2,
            Key::Kp3,
            Key::Kp0,
        ];

        let key_zones = [keys_zone_1.to_vec(), keys_zone_2.to_vec(), keys_zone_3.to_vec(), keys_zone_4.to_vec()];

        let stop_signals = manager.stop_signals.clone();

        let kill_thread = Arc::new(AtomicBool::new(false));
        let exit_thread = kill_thread.clone();

        let mut rx = manager.input_tx.subscribe();

        thread::spawn(move || loop {
            if rx.try_recv().is_ok() && rx.len() == 1 {
                stop_signals.keyboard_stop_signal.store(true, Ordering::SeqCst);
            }

            if exit_thread.load(Ordering::SeqCst) {
                break;
            }

            thread::sleep(Duration::from_millis(5));
        });

        let mut zone_state: [RippleMove; 4] = [RippleMove::Off, RippleMove::Off, RippleMove::Off, RippleMove::Off];

        let mut rx = manager.input_tx.subscribe();

        let mut last_step_time = Instant::now();

        while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
            match rx.try_recv() {
                Ok(event) => {
                    if let rdev::EventType::KeyPress(key) = event.event_type {
                        for (i, zone) in key_zones.iter().enumerate() {
                            if zone.contains(&key) {
                                zone_state[i] = RippleMove::Center;
                            }
                        }
                    }
                }
                Err(err) => {
                    if let tokio::sync::broadcast::error::TryRecvError::Closed = err {
                        break;
                    }
                }
            }

            advance_zone_state(&mut zone_state, &mut last_step_time, &p.speed);

            let rgb_array = p.rgb_array();
            let mut final_arr: [u8; 12] = [0; 12];

            for (i, ripple_move) in zone_state.iter().enumerate() {
                if ripple_move != &RippleMove::Off {
                    final_arr[(i * 3)..((i * 3) + 3)].copy_from_slice(&rgb_array[(i * 3)..((i * 3) + 3)]);
                }
            }

            manager.keyboard.transition_colors_to(&final_arr, 20, 0).unwrap();
            thread::sleep(Duration::from_millis(50));
        }

        kill_thread.store(true, Ordering::SeqCst);
    }
}

fn advance_zone_state(zone_state: &mut [RippleMove; 4], last_step_time: &mut Instant, speed: &u8) {
    let now = Instant::now();
    if now - *last_step_time > Duration::from_millis((200 / *speed) as u64) {
        *last_step_time = now;
        let zone_range = 0..4;
        for (i, ripple_move) in zone_state.clone().iter().enumerate() {
            // Left needs to be signed due to overflows
            let left = i as i32 - 1;
            let right = i + 1;
            match ripple_move {
                RippleMove::Center => {
                    if left >= 0 && zone_range.contains(&(left as usize)) {
                        zone_state[left as usize] = RippleMove::Left;
                    }

                    if zone_range.contains(&right) {
                        zone_state[right] = RippleMove::Right;
                    }
                    zone_state[i] = RippleMove::Off;
                }
                RippleMove::Left => {
                    if zone_range.contains(&(left as usize)) {
                        zone_state[left as usize] = RippleMove::Left;
                    }
                    zone_state[i] = RippleMove::Off;
                }
                RippleMove::Right => {
                    if zone_range.contains(&right) {
                        zone_state[right] = RippleMove::Right;
                    }
                    zone_state[i] = RippleMove::Off;
                }
                _ => {}
            }
        }
    };
}
