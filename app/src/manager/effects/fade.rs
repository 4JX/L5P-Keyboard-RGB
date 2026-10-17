use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

#[cfg(target_os = "windows")]
use device_query::DeviceQuery;

use crate::manager::{profile::Profile, Inner};

pub fn play(manager: &mut Inner, p: &Profile) {
    let stop_signals = manager.stop_signals.clone();

    let kill_thread = Arc::new(AtomicBool::new(false));
    let exit_thread = kill_thread.clone();

    // Shared activity flag for evdev-based input (Linux)
    #[cfg(target_os = "linux")]
    let activity = Arc::new(AtomicBool::new(false));
    #[cfg(target_os = "linux")]
    let activity_writer = activity.clone();

    #[cfg(target_os = "windows")]
    let state = device_query::DeviceState::new();

    #[cfg(target_os = "windows")]
    {
        let stop_signals = stop_signals.clone();
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
    }

    #[cfg(target_os = "linux")]
    {
        let stop_signals = stop_signals.clone();
        thread::spawn(move || {
            let mut dev = match crate::input::find_keyboard_device() {
                Some(d) => d,
                None => {
                    eprintln!("Warning: Could not find keyboard input device for fade effect");
                    return;
                }
            };
            let mut consecutive_errors = 0u32;

            loop {
                if exit_thread.load(Ordering::SeqCst) {
                    break;
                }

                if crate::input::poll_device(&dev, 50) {
                    let fetch_ok = match dev.fetch_events() {
                        Ok(events) => {
                            consecutive_errors = 0;
                            for ev in events {
                                if ev.event_type() == evdev::EventType::KEY && ev.value() == 1 {
                                    activity_writer.store(true, Ordering::SeqCst);
                                    stop_signals.keyboard_stop_signal.store(true, Ordering::SeqCst);
                                }
                            }
                            true
                        }
                        Err(_) => false,
                    };

                    if !fetch_ok {
                        consecutive_errors += 1;
                        if consecutive_errors > 5 {
                            eprintln!("Input device lost, attempting to reopen...");
                            thread::sleep(Duration::from_secs(1));
                            if let Some(d) = crate::input::find_keyboard_device() {
                                dev = d;
                                consecutive_errors = 0;
                                eprintln!("Input device reopened successfully");
                            }
                        }
                    }
                }
            }
        });
    }

    let mut now = Instant::now();
    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        #[cfg(target_os = "windows")]
        let has_activity = !state.get_keys().is_empty();

        #[cfg(target_os = "linux")]
        let has_activity = activity.swap(false, Ordering::SeqCst);

        if !has_activity {
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
