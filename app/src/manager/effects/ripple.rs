use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use device_query::{DeviceEvents, Keycode};

use crate::{
    manager::{effects::zones::KEY_ZONES, Inner},
    profile::Profile,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RippleMove {
    Center,
    Left,
    Right,
    Off,
}

pub fn play(manager: &mut Inner, p: &Profile) {
    // Welcome to the definition of i-don't-know-what-im-doing
    let stop_signals = manager.stop_signals.clone();

    let kill_thread = Arc::new(AtomicBool::new(false));
    let exit_thread = kill_thread.clone();

    enum Event {
        KeyPress(Keycode),
        KeyRelease(Keycode),
    }

    let (tx, rx) = crossbeam_channel::unbounded::<Event>();

    thread::spawn(move || {
        let state = device_query::DeviceState::new();

        // tx_clone.send(Event::KeyPress(Keycode::Meta)).unwrap();
        let tx_clone = tx.clone();

        let guard = state.on_key_down(move |key| {
            stop_signals.keyboard_stop_signal.store(true, Ordering::SeqCst);

            let _ = tx_clone.send(Event::KeyPress(*key));
        });

        let guard2 = state.on_key_up(move |key| {
            let _ = tx.send(Event::KeyRelease(*key));
        });

        loop {
            if exit_thread.load(Ordering::SeqCst) {
                drop(guard);
                drop(guard2);

                break;
            }

            thread::sleep(Duration::from_millis(5));
        }
    });

    let mut zone_pressed: [HashSet<Keycode>; 4] = [HashSet::new(), HashSet::new(), HashSet::new(), HashSet::new()];
    let mut zone_state: [RippleMove; 4] = [RippleMove::Off, RippleMove::Off, RippleMove::Off, RippleMove::Off];

    let mut last_step_time = Instant::now();

    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        match rx.try_recv() {
            Ok(event) => match event {
                Event::KeyPress(key) => {
                    for (i, zone) in KEY_ZONES.iter().enumerate() {
                        if zone.contains(&key) {
                            zone_pressed[i].insert(key);
                        }
                    }

                    manager.stop_signals.keyboard_stop_signal.store(false, Ordering::SeqCst);
                }
                Event::KeyRelease(key) => {
                    for (i, zone) in KEY_ZONES.iter().enumerate() {
                        if zone.contains(&key) {
                            zone_pressed[i].remove(&key);
                        }
                    }
                }
            },
            Err(err) => {
                if let crossbeam_channel::TryRecvError::Disconnected = err {
                    break;
                }
            }
        }

        zone_state = advance_zone_state(zone_state, &mut last_step_time, &p.speed);

        for (i, pressed) in zone_pressed.iter().enumerate() {
            if !pressed.is_empty() {
                zone_state[i] = RippleMove::Center;
            }
        }

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

fn advance_zone_state(zone_state: [RippleMove; 4], last_step_time: &mut Instant, speed: &u8) -> [RippleMove; 4] {
    let now = Instant::now();

    if now - *last_step_time > Duration::from_millis((200 / *speed) as u64) {
        let mut new_state: [RippleMove; 4] = [RippleMove::Off, RippleMove::Off, RippleMove::Off, RippleMove::Off];

        *last_step_time = now;

        // Process moves first, then add centers
        for (i, zone_move) in zone_state.iter().enumerate() {
            match zone_move {
                RippleMove::Left => {
                    if i != 0 {
                        if let Some(left) = new_state.get_mut(i - 1) {
                            *left = RippleMove::Left;
                        }
                    }
                }

                RippleMove::Right => {
                    if let Some(right) = new_state.get_mut(i + 1) {
                        *right = RippleMove::Right;
                    }
                }
                _ => {}
            }
        }

        for (i, ripple_move) in zone_state.iter().enumerate() {
            if let RippleMove::Center = ripple_move {
                if i != 0 {
                    if let Some(left) = new_state.get_mut(i - 1) {
                        *left = RippleMove::Left;
                    }
                }

                if let Some(right) = new_state.get_mut(i + 1) {
                    *right = RippleMove::Right;
                }
            }
        }

        new_state
    } else {
        zone_state
    }
}
