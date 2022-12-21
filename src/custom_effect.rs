use std::{fs, path::Path, sync::atomic::Ordering, thread, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{effects::EffectManager, error};

#[derive(Clone, Deserialize, Serialize)]
struct EffectStep {
	rgb_array: [u8; 12],
	step_type: EffectType,
	speed: u8,
	brightness: u8,
	steps: u8,
	delay_between_steps: u64,
	sleep: u64,
}

#[derive(Clone, Deserialize, Serialize)]
enum EffectType {
	Set,
	Transition,
}

#[derive(Deserialize, Serialize)]
pub struct CustomEffect {
	effect_steps: Vec<EffectStep>,
	should_loop: bool,
}

impl CustomEffect {
	pub fn play(&self, manager: &mut EffectManager) {
		manager.stop_signals.store_false();

		//If loading from the cli, the loop is intentional
		#[allow(clippy::while_immutable_condition)]
		'outer: loop {
			for step in self.effect_steps.clone() {
				manager.keyboard.set_speed(step.speed);
				manager.keyboard.set_brightness(step.brightness);
				match step.step_type {
					EffectType::Set => {
						manager.keyboard.set_colors_to(&step.rgb_array);
					}
					EffectType::Transition => {
						manager.keyboard.transition_colors_to(&step.rgb_array, step.steps, step.delay_between_steps);
					}
				}
				if manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
					break 'outer;
				}
				thread::sleep(Duration::from_millis(step.sleep));
			}
			if !self.should_loop {
				break;
			}
		}
	}

	pub fn from_file(mut path_string: String) -> Result<Self, error::Error> {
		if path_string.rsplit('.').next().map(|ext| ext.eq_ignore_ascii_case("json")) != Some(true) {
			path_string = format!("{}{}", path_string, ".json");
		}
		let path = Path::new(&path_string);
		let full_path = fs::canonicalize(path)?;
		let struct_json = fs::read_to_string(&full_path)?;
		let profile: Self = serde_json::from_str(struct_json.as_str())?;
		Ok(profile)
	}
}
