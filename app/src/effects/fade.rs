use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use device_query::DeviceQuery;

use crate::profile::Profile;

pub fn play(manager: &mut super::Inner, p: &Profile) {
    let stop_signals = manager.stop_signals.clone();

    let kill_thread = Arc::new(AtomicBool::new(false));
    let exit_thread = kill_thread.clone();

    let state = device_query::DeviceState::new();

    thread::spawn(move || {
        let state = device_query::DeviceState::new();

        loop {
            if !state.get_keys().is_empty() {
                stop_signals.keyboard_stop_signal.store(true, Ordering::SeqCst);
            }

            if exit_thread.load(Ordering::SeqCst) {
                break;
            }

            thread::sleep(Duration::from_millis(5));
        }
    });

    let mut now = Instant::now();
    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        if state.get_keys().is_empty() {
            if now.elapsed() > Duration::from_secs(20 / u64::from(p.speed)) {
                manager.keyboard.transition_colors_to(&[0; 12], 230, 3).unwrap();
            } else {
                thread::sleep(Duration::from_millis(20));
            }
        } else {
            manager.keyboard.set_colors_to(&p.rgb_array()).unwrap();
            manager.stop_signals.keyboard_stop_signal.store(false, Ordering::SeqCst);
            now = Instant::now();
        }
    }

    kill_thread.store(true, Ordering::SeqCst);
}
