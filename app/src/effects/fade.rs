use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use tokio::sync::broadcast::error::TryRecvError;

use crate::profile::Profile;

pub(super) struct Fade;

impl Fade {
    pub fn play(manager: &mut super::Inner, p: &Profile) {
        let stop_signals = manager.stop_signals.clone();

        let kill_thread = Arc::new(AtomicBool::new(false));
        let exit_thread = kill_thread.clone();

        let mut rx = manager.input_tx.subscribe();

        thread::spawn(move || loop {
            if rx.try_recv().is_ok() {
                stop_signals.keyboard_stop_signal.store(true, Ordering::SeqCst);
            }

            if exit_thread.load(Ordering::SeqCst) {
                break;
            }

            thread::sleep(Duration::from_millis(5));
        });

        let mut rx = manager.input_tx.subscribe();

        let mut now = Instant::now();
        while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
            match rx.try_recv() {
                Ok(_) => {
                    manager.keyboard.set_colors_to(&p.rgb_array()).unwrap();
                    manager.stop_signals.keyboard_stop_signal.store(false, Ordering::SeqCst);
                    now = Instant::now();
                }
                Err(err) => {
                    match err {
                        TryRecvError::Empty | TryRecvError::Lagged(_) => {
                            // Assume an error means no keys/events
                            if now.elapsed() > Duration::from_secs(20 / u64::from(p.speed)) {
                                manager.keyboard.transition_colors_to(&[0; 12], 230, 3).unwrap();
                            } else {
                                thread::sleep(Duration::from_millis(20));
                            }
                        }
                        TryRecvError::Closed => break,
                    }
                }
            }
        }

        kill_thread.store(true, Ordering::SeqCst);
    }
}
