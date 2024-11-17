use std::{sync::atomic::Ordering, thread, time::Duration};

use rand::Rng;

use crate::profile::Profile;

pub fn play(manager: &mut super::Inner, p: &Profile, thread_rng: &mut rand::rngs::ThreadRng) {
    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let colors = [[255, 0, 0], [255, 255, 0], [0, 255, 0], [0, 255, 255], [0, 0, 255], [255, 0, 255]];
        let colors_index = thread_rng.gen_range(0..6);
        let new_values = colors[colors_index];

        let zone_index = thread_rng.gen_range(0..4);
        manager.keyboard.set_zone_by_index(zone_index, new_values).unwrap();
        thread::sleep(Duration::from_millis(2000 / (u64::from(p.speed) * 4)));
    }
}
