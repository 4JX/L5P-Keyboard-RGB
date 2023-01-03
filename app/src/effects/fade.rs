use std::{
	sync::atomic::Ordering,
	thread,
	time::{Duration, Instant},
};

use device_query::{DeviceQuery, DeviceState, Keycode};

use crate::profile::Profile;

pub(super) struct Fade;

impl Fade {
	pub fn play(manager: &mut super::Inner, p: Profile) {
		let stop_signals = manager.stop_signals.clone();
		thread::spawn(move || {
			let device_state = DeviceState::new();
			while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
				let keys: Vec<Keycode> = device_state.get_keys();
				if !keys.is_empty() {
					stop_signals.keyboard_stop_signal.store(true, Ordering::SeqCst);
				}
				thread::sleep(Duration::from_millis(5));
			}
		});

		let device_state = DeviceState::new();
		let mut now = Instant::now();
		while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
			let keys: Vec<Keycode> = device_state.get_keys();
			if keys.is_empty() {
				if now.elapsed() > Duration::from_secs(20 / u64::from(p.speed)) {
					manager.keyboard.transition_colors_to(&[0; 12], 230, 3).unwrap();
				} else {
					thread::sleep(Duration::from_millis(20));
				}
			} else {
				manager.keyboard.set_colors_to(&p.rgb_array).unwrap();
				manager.stop_signals.keyboard_stop_signal.store(false, Ordering::SeqCst);
				now = Instant::now();
			}
		}
	}
}
