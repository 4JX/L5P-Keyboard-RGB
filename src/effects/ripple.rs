use std::{
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	thread,
	time::{Duration, Instant},
};

use device_query::{DeviceQuery, DeviceState, Keycode};

use crate::profile::Profile;

use super::EffectPlayer;

pub(super) struct Ripple;

impl EffectPlayer for Ripple {
	fn play(manager: &mut super::EffectManager, p: Profile, _thread_rng: &mut rand::rngs::ThreadRng) {
		// Welcome to the definition of i-don't-know-what-im-doing
		let keys_zone_1: [Keycode; 24] = [
			Keycode::Escape,
			Keycode::F1,
			Keycode::F2,
			Keycode::F3,
			Keycode::F4,
			Keycode::Grave,
			Keycode::Key1,
			Keycode::Key2,
			Keycode::Key3,
			Keycode::Key4,
			Keycode::Tab,
			Keycode::Q,
			Keycode::W,
			Keycode::E,
			Keycode::CapsLock,
			Keycode::A,
			Keycode::S,
			Keycode::D,
			Keycode::LShift,
			Keycode::Z,
			Keycode::X,
			Keycode::LControl,
			Keycode::Meta,
			Keycode::LAlt,
		];
		let keys_zone_2: [Keycode; 29] = [
			Keycode::F5,
			Keycode::F6,
			Keycode::F7,
			Keycode::F8,
			Keycode::F9,
			Keycode::F10,
			Keycode::Key5,
			Keycode::Key6,
			Keycode::Key7,
			Keycode::Key8,
			Keycode::Key9,
			Keycode::R,
			Keycode::T,
			Keycode::Y,
			Keycode::U,
			Keycode::I,
			Keycode::F,
			Keycode::G,
			Keycode::H,
			Keycode::J,
			Keycode::K,
			Keycode::C,
			Keycode::V,
			Keycode::B,
			Keycode::N,
			Keycode::M,
			Keycode::Comma,
			Keycode::Space,
			Keycode::RAlt,
		];
		let keys_zone_3: [Keycode; 25] = [
			Keycode::F11,
			Keycode::F12,
			Keycode::Insert,
			Keycode::Delete,
			Keycode::Key0,
			Keycode::Minus,
			Keycode::Equal,
			Keycode::Backspace,
			Keycode::O,
			Keycode::P,
			Keycode::LeftBracket,
			Keycode::RightBracket,
			Keycode::Enter,
			Keycode::L,
			Keycode::Semicolon,
			Keycode::Apostrophe,
			Keycode::BackSlash,
			Keycode::Dot,
			Keycode::Slash,
			Keycode::RShift,
			Keycode::RControl,
			Keycode::Up,
			Keycode::Down,
			Keycode::Left,
			Keycode::Right,
		];

		let keys_zone_4: [Keycode; 18] = [
			Keycode::Home,
			Keycode::End,
			Keycode::PageUp,
			Keycode::PageDown,
			Keycode::NumpadDivide,
			Keycode::NumpadMultiply,
			Keycode::NumpadSubtract,
			Keycode::Numpad7,
			Keycode::Numpad8,
			Keycode::Numpad9,
			Keycode::Numpad4,
			Keycode::Numpad5,
			Keycode::Numpad6,
			Keycode::NumpadAdd,
			Keycode::Numpad1,
			Keycode::Numpad2,
			Keycode::Numpad3,
			Keycode::Numpad0,
		];

		let key_zones = [keys_zone_1.to_vec(), keys_zone_2.to_vec(), keys_zone_3.to_vec(), keys_zone_4.to_vec()];
		let effect_active = Arc::new(AtomicBool::new(false));

		let stop_signals = manager.stop_signals.clone();
		let eff_active = effect_active.clone();
		thread::spawn(move || {
			let device_state = DeviceState::new();
			let mut no_keys_pressed = true;
			while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
				let keys: Vec<Keycode> = device_state.get_keys();
				if !keys.is_empty() && no_keys_pressed {
					stop_signals.keyboard_stop_signal.store(true, Ordering::SeqCst);
					eff_active.store(false, Ordering::SeqCst);
					no_keys_pressed = false
				} else {
					stop_signals.keyboard_stop_signal.store(false, Ordering::SeqCst);
					no_keys_pressed = true
				}
			}
		});

		#[derive(Clone, Copy, PartialEq, Eq)]
		enum RippleMove {
			Center,
			Left,
			Right,
			Off,
		}

		let device_state = DeviceState::new();
		let mut zone_state: [RippleMove; 4] = [RippleMove::Off, RippleMove::Off, RippleMove::Off, RippleMove::Off];
		let mut last_step_time = Instant::now();
		while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
			let keys: Vec<Keycode> = device_state.get_keys();

			if !effect_active.load(Ordering::SeqCst) {
				for (i, zone) in key_zones.iter().enumerate() {
					if keys.iter().any(|key| zone.contains(key)) {
						zone_state[i] = RippleMove::Center
					}
				}

			// thread::sleep(Duration::from_millis(3000));
			// let i = thread_rng.gen_range(0..4);
			// zone_state[i] = RippleMove::Center
			} else {
				let now = Instant::now();
				if now - last_step_time > Duration::from_millis((100.0 / (f32::from(p.speed) / 2.0)) as u64) {
					last_step_time = now;
					let zone_range = 0..4;
					for (i, ripple_move) in zone_state.clone().iter().enumerate() {
						// Left needs to be signed due to overflows
						let left = i as i32 - 1;
						let right = i + 1;
						match ripple_move {
							RippleMove::Center => {
								if left >= 0 && zone_range.contains(&(left as usize)) {
									zone_state[left as usize] = RippleMove::Left
								}

								if zone_range.contains(&right) {
									zone_state[right] = RippleMove::Right
								}
								zone_state[i] = RippleMove::Off;
							}
							RippleMove::Left => {
								if zone_range.contains(&(left as usize)) {
									zone_state[left as usize] = RippleMove::Left
								}
								zone_state[i] = RippleMove::Off;
							}
							RippleMove::Right => {
								if zone_range.contains(&right) {
									zone_state[right] = RippleMove::Right
								}
								zone_state[i] = RippleMove::Off;
							}
							_ => {}
						}
					}
				};
			}

			effect_active.store(false, Ordering::SeqCst);
			let mut final_arr: [u8; 12] = [0; 12];
			for (i, ripple_move) in zone_state.iter().enumerate() {
				if ripple_move != &RippleMove::Off {
					final_arr[i * 3] = p.rgb_array[i * 3];
					final_arr[i * 3 + 1] = p.rgb_array[i * 3 + 1];
					final_arr[i * 3 + 2] = p.rgb_array[i * 3 + 2];
					effect_active.store(true, Ordering::SeqCst);
				}
			}

			manager.keyboard.transition_colors_to(&final_arr, 20, 0);
			thread::sleep(Duration::from_millis(50));
		}
	}
}
