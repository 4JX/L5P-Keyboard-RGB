use std::{sync::atomic::Ordering, thread, time::Duration};

use crate::enums::Direction;

use super::EffectPlayer;

pub(super) struct Swipe;

impl EffectPlayer for Swipe {
	fn play(manager: &mut super::EffectManager, direction: crate::enums::Direction, rgb_array: &[u8; 12], _speed: u8, _brightness: u8, _thread_rng: &mut rand::rngs::ThreadRng) {
		let mut gradient = *rgb_array;

		while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
			if manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
				break;
			}

			for _i in 0..4 {
				match direction {
					Direction::Left => gradient.rotate_right(3),
					Direction::Right => gradient.rotate_left(3),
				}

				manager.keyboard.transition_colors_to(&gradient, 150 / manager.keyboard.get_speed(), 10);
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
