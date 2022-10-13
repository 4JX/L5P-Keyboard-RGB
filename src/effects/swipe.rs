use std::{sync::atomic::Ordering, thread, time::Duration};

use crate::{enums::Direction, profile::Profile};

use super::EffectPlayer;

pub(super) struct Swipe;

impl EffectPlayer for Swipe {
	fn play(manager: &mut super::EffectManager, mut p: Profile, _thread_rng: &mut rand::rngs::ThreadRng) {
		while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
			if manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
				break;
			}

			for _i in 0..4 {
				match p.direction {
					Direction::Left => p.rgb_array.rotate_right(3),
					Direction::Right => p.rgb_array.rotate_left(3),
				}

				manager.keyboard.transition_colors_to(&p.rgb_array, 150 / manager.keyboard.get_speed(), 10);
				if manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
					break;
				}
			}
			if manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
				break;
			}
			thread::sleep(Duration::from_millis(20));
		}
	}
}
