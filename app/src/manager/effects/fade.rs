use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use legion_input::EventKind;

use crate::manager::{profile::Profile, Inner};

pub fn play(manager: &mut Inner, p: &Profile) {
    let Some(input) = manager.input.clone() else {
        return;
    };
    let stop_signals = manager.stop_signals.clone();

    let kill_thread = Arc::new(AtomicBool::new(false));
    let exit_thread = kill_thread.clone();
    let events = input.subscribe();

    let input_thread = thread::spawn(move || {
        while !exit_thread.load(Ordering::SeqCst) {
            if let Ok(event) = events.recv_timeout(Duration::from_millis(50)) {
                if matches!(event.kind, EventKind::KeyDown(_)) {
                    // Interrupt a fade-to-black transition so the effect loop
                    // can restore the profile colors immediately.
                    stop_signals.keyboard_stop_signal.store(true, Ordering::SeqCst);
                }
            }
        }
    });

    let mut now = Instant::now();
    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        if !input.any_pressed() {
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
    let _ = input_thread.join();
}
