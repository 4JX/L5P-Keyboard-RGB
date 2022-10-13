use std::{sync::atomic::Ordering, thread, time::Duration};

use sysinfo::{ComponentExt, System, SystemExt};

use crate::profile::Profile;

use super::EffectPlayer;

pub(super) struct Temperature;

impl EffectPlayer for Temperature {
	fn play(manager: &mut super::EffectManager, _p: Profile, _thread_rng: &mut rand::rngs::ThreadRng) {
		let safe_temp = 30.0;
		let ramp_boost = 1.6;
		let temp_cool: [f32; 12] = [0.0, 255.0, 0.0, 0.0, 255.0, 0.0, 0.0, 255.0, 0.0, 0.0, 255.0, 0.0];
		let temp_hot: [f32; 12] = [255.0, 0.0, 0.0, 255.0, 0.0, 0.0, 255.0, 0.0, 0.0, 255.0, 0.0, 0.0];

		let mut color_differences: [f32; 12] = [0.0; 12];
		for index in 0..12 {
			color_differences[index] = temp_hot[index] - temp_cool[index];
		}

		let mut sys = System::new_all();
		sys.refresh_all();

		for component in sys.components_mut() {
			if component.label() == "Tctl" {
				while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
					component.refresh();
					let mut adjusted_temp = component.temperature() - safe_temp;
					if adjusted_temp < 0.0 {
						adjusted_temp = 0.0;
					}
					let temp_percent = (adjusted_temp / 100.0) * ramp_boost;

					let mut target = [0.0; 12];
					for index in 0..12 {
						target[index] = color_differences[index].mul_add(temp_percent, temp_cool[index]);
					}
					manager.keyboard.transition_colors_to(&target.map(|val| val as u8), 5, 1);
					thread::sleep(Duration::from_millis(20));
				}
			}
		}
	}
}
