use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use legion_input::{Event, EventKind, Key};

use crate::manager::{
    profile::Profile,
    {effects::zones::KEY_ZONES, Inner},
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
    let Some(input) = manager.input.clone() else {
        return;
    };
    let stop_signals = manager.stop_signals.clone();

    let kill_thread = Arc::new(AtomicBool::new(false));
    let exit_thread = kill_thread.clone();
    let (tx, rx) = crossbeam_channel::unbounded::<Event>();
    let events = input.subscribe();

    let input_thread = thread::spawn(move || {
        while !exit_thread.load(Ordering::SeqCst) {
            if let Ok(event) = events.recv_timeout(Duration::from_millis(50)) {
                if matches!(event.kind, EventKind::KeyDown(_)) {
                    stop_signals.keyboard_stop_signal.store(true, Ordering::SeqCst);
                }
                if tx.send(event).is_err() {
                    break;
                }
            }
        }
    });

    let mut zone_pressed: [HashSet<Key>; 4] = [HashSet::new(), HashSet::new(), HashSet::new(), HashSet::new()];
    let mut zone_state: [RippleMove; 4] = [RippleMove::Off, RippleMove::Off, RippleMove::Off, RippleMove::Off];

    let mut last_step_time = Instant::now();

    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        while let Ok(event) = rx.try_recv() {
            match event.kind {
                EventKind::KeyDown(key) => {
                    for (i, zone) in KEY_ZONES.iter().enumerate() {
                        if zone.contains(&key) {
                            zone_pressed[i].insert(key);
                        }
                    }

                    manager.stop_signals.keyboard_stop_signal.store(false, Ordering::SeqCst);
                }
                EventKind::KeyUp(key) => {
                    for (i, zone) in KEY_ZONES.iter().enumerate() {
                        if zone.contains(&key) {
                            zone_pressed[i].remove(&key);
                        }
                    }
                }
                EventKind::Reset => {
                    for pressed in &mut zone_pressed {
                        pressed.clear();
                    }
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
    let _ = input_thread.join();
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
            if matches!(ripple_move, RippleMove::Center) {
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
