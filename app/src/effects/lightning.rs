use std::{sync::atomic::Ordering, thread, time::Duration};

use rand::{rngs::ThreadRng, Rng};

use crate::profile::Profile;

pub(super) struct Lightning;

impl Lightning {
	pub fn play(manager: &mut super::Inner, p: &Profile, thread_rng: &mut ThreadRng) {
		while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
			if manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
				break;
			}
			let zone = thread_rng.gen_range(0..4);
			let steps = thread_rng.gen_range(50..=200);
			manager.keyboard.set_zone_by_index(zone, [255; 3]).unwrap();
			manager.keyboard.transition_colors_to(&[0; 12], steps / p.speed, 5).unwrap();
			let sleep_time = thread_rng.gen_range(100..=2000);
			thread::sleep(Duration::from_millis(sleep_time));
		}
	}
}
