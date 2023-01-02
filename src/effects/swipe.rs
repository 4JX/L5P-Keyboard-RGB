use std::{sync::atomic::Ordering, thread, time::Duration};

use crate::{enums::Direction, profile::Profile};

pub(super) struct Swipe;

impl Swipe {
	pub fn play(manager: &mut super::Inner, mut p: Profile) {
		while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
			if manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
				break;
			}

			match p.direction {
				Direction::Left => p.rgb_array.rotate_right(3),
				Direction::Right => p.rgb_array.rotate_left(3),
			}

			manager.keyboard.transition_colors_to(&p.rgb_array, 150 / p.speed, 10);

			if manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
				break;
			}
			thread::sleep(Duration::from_millis(20));
		}
	}
}
