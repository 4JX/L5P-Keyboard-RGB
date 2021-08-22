use hidapi::{HidApi, HidDevice};

use std::{thread, time};

use std::error::Error;

const DEVICE_INFO: (u16, u16, u16, u16) = (0x048d, 0xc965, 0xff89, 0x00cc);

pub const RGB_RANGE: std::ops::RangeInclusive<f32> = 0.0..=255.0;
const SPEED_RANGE: std::ops::RangeInclusive<u8> = 1..=4;
const BRIGHTNESS_RANGE: std::ops::RangeInclusive<u8> = 1..=2;

pub enum LightingEffects {
	Static,
	Breath,
	Smooth,
	LeftWave,
	RightWave,
}

pub struct LightingState {
	effect_type: LightingEffects,
	speed: u8,
	brightness: u8,
	rgb_values: [f32; 12],
	separate_zones: bool,
}

pub struct Keyboard {
	keyboard_hid: HidDevice,
	current_state: LightingState,
}

#[allow(dead_code)]
impl Keyboard {
	pub fn refresh(&mut self) {
		self.current_state.separate_zones = true;
		let payload = match build_payload(&self.current_state) {
			Ok(payload) => payload,
			Err(err) => panic!("{}", err),
		};
		match self.keyboard_hid.send_feature_report(&payload) {
			Ok(_keyboard_hid) => {}
			Err(err) => panic!("{}", err),
		};
	}

	pub fn solid_refresh(&mut self) {
		self.current_state.separate_zones = false;
		let payload = match build_payload(&self.current_state) {
			Ok(payload) => payload,
			Err(err) => panic!("{}", err),
		};
		match self.keyboard_hid.send_feature_report(&payload) {
			Ok(_keyboard_hid) => {}
			Err(err) => panic!("{}", err),
		};
	}

	pub fn set_effect(&mut self, effect: LightingEffects) {
		self.current_state.effect_type = effect;
		self.refresh();
	}

	pub fn set_speed(&mut self, speed: u8) {
		if !SPEED_RANGE.contains(&speed) {
			panic!("Speed is outside valid range (1-4)");
		}
		self.current_state.speed = speed;
		self.refresh();
	}

	pub fn set_brightness(&mut self, brightness: u8) {
		if !BRIGHTNESS_RANGE.contains(&brightness) {
			panic!("Brightness is outside valid range (1-2)");
		}
		self.current_state.brightness = brightness;
		self.refresh();
	}

	pub fn set_value_by_index(&mut self, triplet_index: u8, color_index: u8, new_value: f32) {
		if !(0..4).contains(&triplet_index) {
			panic!("Triplet index is outside valid range (0-3)");
		}

		if !(0..3).contains(&color_index) {
			panic!("Color index is outside valid range (0-2)");
		}
		if !RGB_RANGE.contains(&new_value) {
			panic!("Keyboard colors has value outside accepted range (0-255)");
		}
		let full_index: usize = ((triplet_index * 3) + color_index) as usize;
		self.current_state.rgb_values[full_index] = new_value;
		self.refresh();
	}
	pub fn solid_set_value_by_index(&mut self, color_index: u8, new_value: f32) {
		if !(0..3).contains(&color_index) {
			panic!("Color index is outside valid range (0-2)");
		}
		if !RGB_RANGE.contains(&new_value) {
			panic!("Keyboard colors has value outside accepted range (0-255)");
		}
		for i in 0..4 {
			let full_index: usize = ((i * 3) + color_index) as usize;
			self.current_state.rgb_values[full_index] = new_value;
		}
		self.refresh();
	}

	pub fn set_zone_by_index(&mut self, zone_index: u8, new_values: [f32; 3]) {
		if !(0..4).contains(&zone_index) {
			panic!("Zone index is outside valid range (0-3)");
		}
		for val in new_values.iter() {
			if !RGB_RANGE.contains(&val) {
				panic!("Keyboard colors has value outside accepted range (0-255)");
			}
		}
		for (i, _) in new_values.iter().enumerate() {
			let full_index = (zone_index * 3 + i as u8) as usize;
			self.current_state.rgb_values[full_index] = new_values[i];
		}
		self.refresh();
	}

	pub fn set_colors_to(&mut self, rgb_values: &[f32; 12]) {
		match self.current_state.effect_type {
			LightingEffects::Static | LightingEffects::Breath => {
				for i in rgb_values {
					if !RGB_RANGE.contains(i) {
						panic!("Keyboard colors has value outside accepted range (0-255)");
					}
				}
				self.current_state.rgb_values = *rgb_values;
				self.refresh();
			}
			_ => {}
		}
	}

	pub fn transition_colors_to(&mut self, target_colors: &[f32; 12], steps: u8, delay_between_steps: u64) {
		for i in target_colors {
			if !RGB_RANGE.contains(i) {
				panic!("Keyboard colors has value outside accepted range (0-255)");
			}
		}
		match self.current_state.effect_type {
			LightingEffects::Static | LightingEffects::Breath => {
				let mut color_differences: [f32; 12] = [0.0; 12];
				for index in 0..12 {
					color_differences[index] = (target_colors[index] - self.current_state.rgb_values[index]) / steps as f32
				}
				for _step_num in 1..=steps {
					for (index, _) in color_differences.iter().enumerate() {
						self.current_state.rgb_values[index] += color_differences[index];
						if self.current_state.rgb_values[index] > 255.0 {
							self.current_state.rgb_values[index] = 255.0
						} else if self.current_state.rgb_values[index] < 0.0 {
							self.current_state.rgb_values[index] = 0.0
						}
					}
					self.refresh();
					thread::sleep(time::Duration::from_millis(delay_between_steps));
				}
				self.set_colors_to(target_colors)
			}
			_ => {}
		}
	}
}

pub fn get_keyboard() -> Result<Keyboard, Box<dyn Error>> {
	let api: HidApi = HidApi::new()?;

	let info = api
		.device_list()
		.find(|d| (d.vendor_id(), d.product_id(), d.usage_page(), d.usage()) == DEVICE_INFO)
		.ok_or("Error: Couldn't find device")?;

	let keyboard_hid: HidDevice = info.open_device(&api)?;
	let current_state: LightingState = LightingState {
		effect_type: LightingEffects::Static,
		speed: 1,
		brightness: 1,
		rgb_values: [0.0; 12],
		separate_zones: false,
	};

	let mut keyboard = Keyboard { keyboard_hid, current_state };

	keyboard.refresh();
	Ok(keyboard)
}

fn build_payload(keyboard_state: &LightingState) -> Result<[u8; 33], &'static str> {
	if !SPEED_RANGE.contains(&keyboard_state.speed) {
		return Err("Speed is outside valid range (1-4)");
	}
	if !BRIGHTNESS_RANGE.contains(&keyboard_state.brightness) {
		return Err("Brightness is outside valid range (1-2)");
	}
	for i in keyboard_state.rgb_values.iter() {
		if !RGB_RANGE.contains(&i) {
			return Err("Keyboard colors has value outside accepted range (0-255)");
		}
	}
	let mut payload: [u8; 33] = [0; 33];
	payload[0] = 0xcc;
	payload[1] = 0x16;
	payload[2] = match keyboard_state.effect_type {
		LightingEffects::Static => 0x01,
		LightingEffects::Breath => 0x03,
		LightingEffects::Smooth => 0x06,
		LightingEffects::LeftWave => {
			payload[19] = 0x1;
			0x04
		}
		LightingEffects::RightWave => {
			payload[18] = 0x1;
			0x04
		}
	};

	payload[3] = keyboard_state.speed;
	payload[4] = keyboard_state.brightness;

	match keyboard_state.effect_type {
		LightingEffects::Static | LightingEffects::Breath => {
			if keyboard_state.separate_zones {
				for i in 0..12 {
					payload[i + 5] = keyboard_state.rgb_values[i] as u8;
				}
			} else {
				for i in 0..3 {
					payload[i + 5] = keyboard_state.rgb_values[i] as u8;
					payload[i + 8] = keyboard_state.rgb_values[i] as u8;
					payload[i + 11] = keyboard_state.rgb_values[i] as u8;
					payload[i + 14] = keyboard_state.rgb_values[i] as u8;
				}
			}
		}
		_ => {}
	};
	Ok(payload)
}
