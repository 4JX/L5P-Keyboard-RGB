use std::{
	sync::atomic::Ordering,
	thread,
	time::{Duration, Instant},
};

use device_query::{DeviceQuery, DeviceState, Keycode};

use super::EffectPlayer;

pub(super) struct Fade;

impl EffectPlayer for Fade {
	fn play(manager: &mut super::EffectManager, _direction: crate::enums::Direction, rgb_array: &[u8; 12], _speed: u8, _brightness: u8, _thread_rng: &mut rand::rngs::ThreadRng) {
		let stop_signals = manager.stop_signals.clone();
		thread::spawn(move || {
			let device_state = DeviceState::new();
			while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
				let keys: Vec<Keycode> = device_state.get_keys();
				if !keys.is_empty() {
					stop_signals.keyboard_stop_signal.store(true, Ordering::SeqCst);
				}
			}
		});

		let device_state = DeviceState::new();
		let mut now = Instant::now();
		while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
			let keys: Vec<Keycode> = device_state.get_keys();
			if keys.is_empty() {
				if now.elapsed() > Duration::from_secs(20 / u64::from(manager.keyboard.get_speed())) {
					manager.keyboard.transition_colors_to(&[0; 12], 230, 3);
				} else {
					thread::sleep(Duration::from_millis(20));
				}
			} else {
				manager.keyboard.set_colors_to(rgb_array);
				manager.stop_signals.keyboard_stop_signal.store(false, Ordering::SeqCst);
				now = Instant::now();
			}
		}
	}
}
