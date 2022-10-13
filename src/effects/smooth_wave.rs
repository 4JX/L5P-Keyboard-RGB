use std::{sync::atomic::Ordering, thread, time::Duration};

use crate::{enums::Direction, profile::Profile};

use super::EffectPlayer;

pub(super) struct SmoothWave;

impl EffectPlayer for SmoothWave {
	fn play(manager: &mut super::EffectManager, p: Profile, _thread_rng: &mut rand::rngs::ThreadRng) {
		let mut gradient = [255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 255];

		while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
			if manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
				break;
			}
			match p.direction {
				Direction::Left => gradient.rotate_right(3),
				Direction::Right => gradient.rotate_left(3),
			}
			manager.keyboard.transition_colors_to(&gradient, 70 / manager.keyboard.get_speed(), 10);
			if manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
				break;
			}
			thread::sleep(Duration::from_millis(20));
		}
	}
}
