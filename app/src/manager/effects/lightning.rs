use std::{sync::atomic::Ordering, thread, time::Duration};

use rand::{rngs::ThreadRng, Rng};

use crate::{manager::Inner, profile::Profile};

pub fn play(manager: &mut Inner, p: &Profile, thread_rng: &mut ThreadRng) {
    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let profile_array = p.rgb_array();

        if manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
            break;
        }
        let zone_index = thread_rng.gen_range(0..4);
        let steps = thread_rng.gen_range(50..=200);

        let mut arr = [0; 12];
        let zone_start = zone_index * 3;

        arr[zone_start] = profile_array[zone_start];
        arr[zone_start + 1] = profile_array[zone_start + 1];
        arr[zone_start + 2] = profile_array[zone_start + 2];

        manager.keyboard.set_colors_to(&arr).unwrap();
        manager.keyboard.transition_colors_to(&[0; 12], steps / p.speed, 5).unwrap();
        let sleep_time = thread_rng.gen_range(100..=2000);
        thread::sleep(Duration::from_millis(sleep_time));
    }
}
