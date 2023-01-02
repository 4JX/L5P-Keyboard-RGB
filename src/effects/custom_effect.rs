use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::error;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct EffectStep {
	pub rgb_array: [u8; 12],
	pub step_type: EffectType,
	pub speed: u8,
	pub brightness: u8,
	pub steps: u8,
	pub delay_between_steps: u64,
	pub sleep: u64,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum EffectType {
	Set,
	Transition,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CustomEffect {
	pub effect_steps: Vec<EffectStep>,
	pub should_loop: bool,
}

impl CustomEffect {
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
