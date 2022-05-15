use crate::{
	enums::{Direction, Effects, Message},
	keyboard_utils,
};
use crate::{
	error,
	keyboard_utils::{BaseEffects, Keyboard},
};
use device_query::{DeviceQuery, DeviceState, Keycode};
use fast_image_resize as fr;
use flume::{Receiver, Sender};
use rand::{thread_rng, Rng};
use scrap::{Capturer, Display};
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::{ComponentExt, System, SystemExt};

pub struct KeyboardManager {
	pub keyboard: Keyboard,
	pub rx: Receiver<Message>,
	pub tx: Sender<Message>,
	pub stop_signals: StopSignals,
	pub last_effect: Effects,
}

impl KeyboardManager {
	pub fn new() -> Result<Self, error::Error> {
		let keyboard_stop_signal = Arc::new(AtomicBool::new(false));
		let keyboard = keyboard_utils::get_keyboard(keyboard_stop_signal.clone())?;

		let (tx, rx) = flume::unbounded::<Message>();

		let manager = Self {
			keyboard,
			rx,
			tx,
			stop_signals: StopSignals {
				manager_stop_signal: Arc::new(AtomicBool::new(false)),
				keyboard_stop_signal,
			},
			last_effect: Effects::Static,
		};

		Ok(manager)
	}

	pub fn set_effect(&mut self, effect: Effects, direction: Direction, rgb_array: &[u8; 12], speed: u8, brightness: u8) {
		self.stop_signals.store_false();
		self.last_effect = effect;
		let mut thread_rng = thread_rng();

		self.keyboard.set_effect(BaseEffects::Static);
		self.keyboard.set_speed(speed);
		self.keyboard.set_brightness(brightness);

		match effect {
			Effects::Static => {
				self.keyboard.set_colors_to(rgb_array);
				self.keyboard.set_effect(BaseEffects::Static);
			}
			Effects::Breath => {
				self.keyboard.set_colors_to(rgb_array);
				self.keyboard.set_effect(BaseEffects::Breath);
			}
			Effects::Smooth => {
				self.keyboard.set_effect(BaseEffects::Smooth);
			}
			Effects::Wave => match direction {
				Direction::Left => self.keyboard.set_effect(BaseEffects::LeftWave),
				Direction::Right => self.keyboard.set_effect(BaseEffects::RightWave),
			},

			Effects::Lightning => {
				while !self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
					if self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
						break;
					}
					let zone = thread_rng.gen_range(0..4);
					let steps = thread_rng.gen_range(50..=200);
					self.keyboard.set_zone_by_index(zone, [255; 3]);
					self.keyboard.transition_colors_to(&[0; 12], steps / self.keyboard.get_speed(), 5);
					let sleep_time = thread_rng.gen_range(100..=2000);
					thread::sleep(Duration::from_millis(sleep_time));
				}
			}
			Effects::AmbientLight => {
				//Display setup
				let display = Display::all().unwrap().remove(0);

				let mut capturer = Capturer::new(display, false).expect("Couldn't begin capture.");
				let (w, h) = (capturer.width(), capturer.height());

				let seconds_per_frame = Duration::from_nanos(1_000_000_000 / 60);
				let wait_base: i32 = seconds_per_frame.as_millis() as i32;
				let mut wait = wait_base;
				let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3));

				while !self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
					if self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
						break;
					}

					let now = Instant::now();
					match capturer.frame(wait as u32) {
						Ok(frame) => {
							// Adapted from https://github.com/Cykooz/fast_image_resize#resize-image
							// Read source image from file
							let width = NonZeroU32::new(w as u32).unwrap();
							let height = NonZeroU32::new(h as u32).unwrap();
							let mut src_image = fr::Image::from_vec_u8(width, height, frame.to_vec(), fr::PixelType::U8x4).unwrap();

							// Create MulDiv instance
							let alpha_mul_div: fr::MulDiv = fr::MulDiv::default();
							// Multiple RGB channels of source image by alpha channel
							alpha_mul_div.multiply_alpha_inplace(&mut src_image.view_mut()).unwrap();

							// Create container for data of destination image
							let dst_width = NonZeroU32::new(4).unwrap();
							let dst_height = NonZeroU32::new(1).unwrap();
							let mut dst_image = fr::Image::new(dst_width, dst_height, fr::PixelType::U8x4);

							// Get mutable view of destination image data
							let mut dst_view = dst_image.view_mut();

							// Create Resizer instance and resize source image
							// into buffer of destination image
							resizer.resize(&src_image.view(), &mut dst_view).unwrap();

							// Divide RGB channels of destination image by alpha
							alpha_mul_div.divide_alpha_inplace(&mut dst_view).unwrap();

							let bgr_arr = dst_image.buffer();

							// BGRA -> RGB
							let mut rgb: [u8; 12] = [0; 12];
							for (src, dst) in bgr_arr.chunks_exact(4).zip(rgb.chunks_exact_mut(3)) {
								dst[0] = src[2];
								dst[1] = src[1];
								dst[2] = src[0];
							}

							self.keyboard.set_colors_to(&rgb);
							let elapsed_time = now.elapsed();
							if elapsed_time < seconds_per_frame {
								thread::sleep(seconds_per_frame - elapsed_time);
							}
						}
						Err(error) => match error.kind() {
							std::io::ErrorKind::WouldBlock => {
								wait = wait_base - now.elapsed().as_millis() as i32;
								if wait < 0 {
									wait = 0;
								}
							}
							std::io::ErrorKind::InvalidData => {
								self.stop_signals.store_true();
								self.tx.send(Message::Refresh).unwrap();
							}

							_ => {}
						},
					}
				}
			}
			Effects::SmoothWave => {
				let mut gradient = [255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 255];

				while !self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
					if self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
						break;
					}
					match direction {
						Direction::Left => gradient.rotate_right(3),
						Direction::Right => gradient.rotate_left(3),
					}
					self.keyboard.transition_colors_to(&gradient, 70 / self.keyboard.get_speed(), 10);
					if self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
						break;
					}
					thread::sleep(Duration::from_millis(20));
				}
			}
			Effects::Swipe => {
				let mut gradient = *rgb_array;

				while !self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
					if self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
						break;
					}

					for _i in 0..4 {
						match direction {
							Direction::Left => gradient.rotate_right(3),
							Direction::Right => gradient.rotate_left(3),
						}

						self.keyboard.transition_colors_to(&gradient, 150 / self.keyboard.get_speed(), 10);
						if self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
							break;
						}
					}
					if self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
						break;
					}
					thread::sleep(Duration::from_millis(20));
				}
			}
			Effects::Disco => {
				while !self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
					let colors = [[255, 0, 0], [255, 255, 0], [0, 255, 0], [0, 255, 255], [0, 0, 255], [255, 0, 255]];
					let colors_index = thread_rng.gen_range(0..6);
					let new_values = colors[colors_index];

					let zone_index = thread_rng.gen_range(0..4);
					self.keyboard.set_zone_by_index(zone_index, new_values);
					thread::sleep(Duration::from_millis(2000 / (u64::from(self.keyboard.get_speed()) * 4)));
				}
			}
			Effects::Christmas => {
				let xmas_color_array = [[255, 10, 10], [255, 255, 20], [30, 255, 30], [70, 70, 255]];
				let subeffect_count = 4;
				let mut last_subeffect = -1;
				while !self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
					let mut subeffect = thread_rng.gen_range(0..subeffect_count);
					while last_subeffect == subeffect {
						subeffect = thread_rng.gen_range(0..subeffect_count);
					}
					last_subeffect = subeffect;

					match subeffect {
						0 => {
							for _i in 0..3 {
								for colors in xmas_color_array {
									self.keyboard.solid_set_colors_to(colors);
									thread::sleep(Duration::from_millis(500));
								}
							}
						}
						1 => {
							let color_1_index = thread_rng.gen_range(0..4);
							let used_colors_1: [u8; 3] = xmas_color_array[color_1_index];

							let mut color_2_index = thread_rng.gen_range(0..4);
							while color_1_index == color_2_index {
								color_2_index = thread_rng.gen_range(0..4);
							}
							let used_colors_2: [u8; 3] = xmas_color_array[color_2_index];

							for _i in 0..4 {
								self.keyboard.solid_set_colors_to(used_colors_1);
								thread::sleep(Duration::from_millis(400));
								self.keyboard.solid_set_colors_to(used_colors_2);
								thread::sleep(Duration::from_millis(400));
							}
						}
						2 => {
							let steps = 100;
							self.keyboard.transition_colors_to(&[0; 12], steps, 1);
							let mut used_colors_array: [u8; 12] = [0; 12];
							let left_or_right = thread_rng.gen_range(0..2);
							if left_or_right == 0 {
								for color in xmas_color_array {
									for j in (0..12).step_by(3) {
										used_colors_array[j] = color[0];
										used_colors_array[j + 1] = color[1];
										used_colors_array[j + 2] = color[2];
										self.keyboard.transition_colors_to(&used_colors_array, steps, 1);
									}
									for j in (0..12).step_by(3) {
										used_colors_array[j] = 0;
										used_colors_array[j + 1] = 0;
										used_colors_array[j + 2] = 0;
										self.keyboard.transition_colors_to(&used_colors_array, steps, 1);
									}
								}
							} else {
								for i in 0..4 {
									for j in (0..12).step_by(3) {
										used_colors_array[11 - j] = xmas_color_array[3 - i][0];
										used_colors_array[11 - (j + 1)] = xmas_color_array[3 - i][1];
										used_colors_array[11 - (j + 2)] = xmas_color_array[3 - i][2];
										self.keyboard.transition_colors_to(&used_colors_array, steps, 1);
									}
									for j in (0..12).step_by(3) {
										used_colors_array[11 - j] = 0;
										used_colors_array[11 - (j + 1)] = 0;
										used_colors_array[11 - (j + 2)] = 0;
										self.keyboard.transition_colors_to(&used_colors_array, steps, 1);
									}
								}
							}
						}
						3 => {
							let state1 = [255, 255, 255, 0, 0, 0, 255, 255, 255, 0, 0, 0];
							let state2 = [0, 0, 0, 255, 255, 255, 0, 0, 0, 255, 255, 255];
							let steps = 30;
							for _i in 0..4 {
								self.keyboard.transition_colors_to(&state1, steps, 1);
								thread::sleep(Duration::from_millis(400));
								self.keyboard.transition_colors_to(&state2, steps, 1);
								thread::sleep(Duration::from_millis(400));
							}
						}
						_ => unreachable!("Subeffect index for Christmas effect is out of range."),
					}
				}
			}
			Effects::Fade => {
				let stop_signals = self.stop_signals.clone();
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
				while !self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
					let keys: Vec<Keycode> = device_state.get_keys();
					if keys.is_empty() {
						if now.elapsed() > Duration::from_secs(20 / u64::from(self.keyboard.get_speed())) {
							self.keyboard.transition_colors_to(&[0; 12], 230, 3);
						} else {
							thread::sleep(Duration::from_millis(20));
						}
					} else {
						self.keyboard.set_colors_to(rgb_array);
						self.stop_signals.keyboard_stop_signal.store(false, Ordering::SeqCst);
						now = Instant::now();
					}
				}
			}
			Effects::Temperature => {
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
						while !self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
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
							self.keyboard.transition_colors_to(&target.map(|val| val as u8), 5, 1);
							thread::sleep(Duration::from_millis(20));
						}
					}
				}
			}
			Effects::Ripple => {
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

				let stop_signals = self.stop_signals.clone();
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
				while !self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
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
						if now - last_step_time > Duration::from_millis((100.0 / (f32::from(speed) / 2.0)) as u64) {
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
							final_arr[i * 3] = rgb_array[i * 3];
							final_arr[i * 3 + 1] = rgb_array[i * 3 + 1];
							final_arr[i * 3 + 2] = rgb_array[i * 3 + 2];
							effect_active.store(true, Ordering::SeqCst);
						}
					}

					self.keyboard.transition_colors_to(&final_arr, 20, 0);
					thread::sleep(Duration::from_millis(50));
				}
			}
		}
		self.stop_signals.store_false();
	}
}

#[derive(Clone)]
pub struct StopSignals {
	pub manager_stop_signal: Arc<AtomicBool>,
	pub keyboard_stop_signal: Arc<AtomicBool>,
}

impl StopSignals {
	pub fn store_true(&self) {
		self.keyboard_stop_signal.store(true, Ordering::SeqCst);
		self.manager_stop_signal.store(true, Ordering::SeqCst);
	}
	pub fn store_false(&self) {
		self.keyboard_stop_signal.store(false, Ordering::SeqCst);
		self.manager_stop_signal.store(false, Ordering::SeqCst);
	}
}
