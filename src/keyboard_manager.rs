use crate::enums::{Effects, Message};
use crate::keyboard_utils::{BaseEffects, Keyboard};

use image::{buffer::ConvertBuffer, imageops, ImageBuffer};
use rand::{thread_rng, Rng};
use scrap::{Capturer, Display};
use std::convert::TryInto;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub struct KeyboardManager {
	pub keyboard: Keyboard,
	pub rx: Receiver<Message>,
	pub stop_signal: Arc<AtomicBool>,
	pub last_effect: Effects,
}

impl KeyboardManager {
	pub fn set_effect(&mut self, effect: Effects, color_array: &[u8; 12], speed: u8, brightness: u8) {
		self.stop_signal.store(false, Ordering::SeqCst);
		self.last_effect = effect;
		let mut thread_rng = thread_rng();

		self.keyboard.set_effect(BaseEffects::Static);
		self.keyboard.current_state.speed = speed;
		self.keyboard.current_state.brightness = brightness;

		match effect {
			Effects::Static => {
				self.keyboard.set_colors_to(color_array);
				self.keyboard.set_effect(BaseEffects::Static);
			}
			Effects::Breath => {
				self.keyboard.set_colors_to(color_array);
				self.keyboard.set_effect(BaseEffects::Breath);
			}
			Effects::Smooth => {
				self.keyboard.set_effect(BaseEffects::Smooth);
			}
			Effects::LeftWave => {
				self.keyboard.set_effect(BaseEffects::LeftWave);
			}
			Effects::RightWave => {
				self.keyboard.set_effect(BaseEffects::RightWave);
			}
			Effects::Lightning => {
				while !self.stop_signal.load(Ordering::SeqCst) {
					if self.stop_signal.load(Ordering::SeqCst) {
						break;
					}
					let zone = thread_rng.gen_range(0..4);
					let steps = thread_rng.gen_range(50..=200);
					self.keyboard.set_zone_by_index(zone, [255; 3]);
					self.keyboard.transition_colors_to(&[0.0; 12], steps / self.keyboard.current_state.speed, 5);
					let sleep_time = thread_rng.gen_range(100..=2000);
					thread::sleep(Duration::from_millis(sleep_time));
				}
			}
			Effects::AmbientLight => {
				//Display setup
				let displays = Display::all().unwrap().len();
				for i in 0..displays {
					type BgraImage<V> = ImageBuffer<image::Bgra<u8>, V>;
					let display = Display::all().unwrap().remove(i);

					let mut capturer = Capturer::new(display, false).expect("Couldn't begin capture.");
					let (w, h) = (capturer.width(), capturer.height());

					let seconds_per_frame = Duration::from_nanos(1_000_000_000 / 30);
					while !self.stop_signal.load(Ordering::SeqCst) {
						if self.stop_signal.load(Ordering::SeqCst) {
							break;
						}
						if let Ok(frame) = capturer.frame(0) {
							let now = Instant::now();
							let bgra_img = BgraImage::from_raw(w as u32, h as u32, &*frame).expect("Could not get bgra image.");
							let rgb_img: image::RgbImage = bgra_img.convert();
							let resized = imageops::resize(&rgb_img, 4, 1, imageops::FilterType::Lanczos3);
							let dst = resized.into_vec();

							let mut result: [f32; 12] = [0.0; 12];
							for i in 0..12 {
								result[i] = f32::from(dst[i]);
							}
							self.keyboard.transition_colors_to(&result, 4, 1);
							let elapsed_time = now.elapsed();
							if elapsed_time < seconds_per_frame {
								thread::sleep(seconds_per_frame - elapsed_time);
							}
						} else {
							//Janky recover from error because it does not like admin prompts on windows
							let displays = Display::all().unwrap().len();
							for i in 0..displays {
								let display = Display::all().unwrap().remove(i);

								capturer = Capturer::new(display, false).expect("Couldn't begin capture.");
							}
						}
						thread::sleep(Duration::from_millis(20));
					}
					drop(capturer);
				}
			}
			Effects::SmoothLeftWave => {
				let mut gradient = vec![255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 255.0, 255.0, 0.0, 255.0];

				while !self.stop_signal.load(Ordering::SeqCst) {
					if self.stop_signal.load(Ordering::SeqCst) {
						break;
					}
					shift_vec(&mut gradient, 3);
					let colors: [f32; 12] = gradient.clone().try_into().unwrap();
					self.keyboard.transition_colors_to(&colors, 70 / self.keyboard.current_state.speed, 10);
					if self.stop_signal.load(Ordering::SeqCst) {
						break;
					}
					thread::sleep(Duration::from_millis(20));
				}
			}
			Effects::SmoothRightWave => {
				let mut gradient = vec![255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 255.0, 255.0, 0.0, 255.0];

				while !self.stop_signal.load(Ordering::SeqCst) {
					if self.stop_signal.load(Ordering::SeqCst) {
						break;
					}
					shift_vec(&mut gradient, 9);
					let colors: [f32; 12] = gradient.clone().try_into().unwrap();
					self.keyboard.transition_colors_to(&colors, 70 / self.keyboard.current_state.speed, 10);
					if self.stop_signal.load(Ordering::SeqCst) {
						break;
					}
					thread::sleep(Duration::from_millis(20));
				}
			}
			Effects::LeftSwipe => {
				while !self.stop_signal.load(Ordering::SeqCst) {
					if self.stop_signal.load(Ordering::SeqCst) {
						break;
					}

					let mut gradient = color_array.map(f32::from).to_vec();
					for _i in 0..4 {
						shift_vec(&mut gradient, 3);
						let colors: [f32; 12] = gradient.clone().try_into().unwrap();
						self.keyboard.transition_colors_to(&colors, 150 / self.keyboard.current_state.speed, 10);
						if self.stop_signal.load(Ordering::SeqCst) {
							break;
						}
					}
					if self.stop_signal.load(Ordering::SeqCst) {
						break;
					}
					thread::sleep(Duration::from_millis(20));
				}
			}
			Effects::RightSwipe => {
				while !self.stop_signal.load(Ordering::SeqCst) {
					if self.stop_signal.load(Ordering::SeqCst) {
						break;
					}

					let mut gradient = color_array.map(f32::from).to_vec();
					for _i in 0..4 {
						shift_vec(&mut gradient, 9);
						let colors: [f32; 12] = gradient.clone().try_into().unwrap();
						self.keyboard.transition_colors_to(&colors, 150 / self.keyboard.current_state.speed, 10);
						if self.stop_signal.load(Ordering::SeqCst) {
							break;
						}
					}
					if self.stop_signal.load(Ordering::SeqCst) {
						break;
					}
					thread::sleep(Duration::from_millis(20));
				}
			}
			Effects::Disco => {
				while !self.stop_signal.load(Ordering::SeqCst) {
					let colors = [[255, 0, 0], [255, 255, 0], [0, 255, 0], [0, 255, 255], [0, 0, 255], [255, 0, 255]];
					let colors_index = thread_rng.gen_range(0..6);
					let new_values = colors[colors_index];

					let zone_index = thread_rng.gen_range(0..4);
					self.keyboard.set_zone_by_index(zone_index, new_values);
					thread::sleep(Duration::from_millis(2000 / (u64::from(self.keyboard.current_state.speed) * 4)));
				}
			}
			Effects::Christmas => {
				let color_array = [255, 10, 10, 255, 255, 20, 30, 255, 30, 70, 70, 255];
				let subeffect_count = 4;
				let mut last_subeffect = -1;
				while !self.stop_signal.load(Ordering::SeqCst) {
					let mut subeffect = thread_rng.gen_range(0..subeffect_count);
					while last_subeffect == subeffect {
						subeffect = thread_rng.gen_range(0..subeffect_count);
					}
					last_subeffect = subeffect;

					match subeffect {
						0 => {
							for _i in 0..3 {
								for j in 0..4 {
									let array_index = j * 3;
									let used_colors: [u8; 3] = color_array[array_index..array_index + 3].try_into().unwrap();
									self.keyboard.solid_set_colors_to(&used_colors);
									thread::sleep(Duration::from_millis(500));
								}
							}
						}
						1 => {
							let cut_start_1 = (thread_rng.gen_range(0..4)) * 3;
							let used_colors_1: [u8; 3] = color_array[cut_start_1..cut_start_1 + 3].try_into().unwrap();

							let mut cut_start_2 = (thread_rng.gen_range(0..4)) * 3;
							while cut_start_1 == cut_start_2 {
								cut_start_2 = (thread_rng.gen_range(0..4)) * 3;
							}
							let used_colors_2: [u8; 3] = color_array[cut_start_2..cut_start_2 + 3].try_into().unwrap();

							for _i in 0..4 {
								self.keyboard.solid_set_colors_to(&used_colors_1);
								thread::sleep(Duration::from_millis(400));
								self.keyboard.solid_set_colors_to(&used_colors_2);
								thread::sleep(Duration::from_millis(400));
							}
						}
						2 => {
							let steps = 100;
							self.keyboard.transition_colors_to(&[0.0; 12], steps, 1);
							let mut used_colors_array: [f32; 12] = [0.0; 12];
							let left_or_right = thread_rng.gen_range(0..2);
							if left_or_right == 0 {
								for i in (0..12).step_by(3) {
									for j in (0..12).step_by(3) {
										used_colors_array[j] = f32::from(color_array[i]);
										used_colors_array[j + 1] = f32::from(color_array[i + 1]);
										used_colors_array[j + 2] = f32::from(color_array[i + 2]);
										self.keyboard.transition_colors_to(&used_colors_array, steps, 1);
									}
									for j in (0..12).step_by(3) {
										used_colors_array[j] = 0.0;
										used_colors_array[j + 1] = 0.0;
										used_colors_array[j + 2] = 0.0;
										self.keyboard.transition_colors_to(&used_colors_array, steps, 1);
									}
								}
							} else {
								for i in (0..12).step_by(3) {
									for j in (0..12).step_by(3) {
										used_colors_array[11 - j] = f32::from(color_array[11 - i]);
										used_colors_array[11 - (j + 1)] = f32::from(color_array[11 - (i + 1)]);
										used_colors_array[11 - (j + 2)] = f32::from(color_array[11 - (i + 2)]);
										self.keyboard.transition_colors_to(&used_colors_array, steps, 1);
									}
									for j in (0..12).step_by(3) {
										used_colors_array[11 - j] = 0.0;
										used_colors_array[11 - (j + 1)] = 0.0;
										used_colors_array[11 - (j + 2)] = 0.0;
										self.keyboard.transition_colors_to(&used_colors_array, steps, 1);
									}
								}
							}
						}
						3 => {
							let state1 = [255.0, 255.0, 255.0, 0.0, 0.0, 0.0, 255.0, 255.0, 255.0, 0.0, 0.0, 0.0];
							let state2 = [0.0, 0.0, 0.0, 255.0, 255.0, 255.0, 0.0, 0.0, 0.0, 255.0, 255.0, 255.0];
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
		}
		self.stop_signal.store(false, Ordering::SeqCst);
	}
}

fn shift_vec(vec: &mut Vec<f32>, steps: u8) {
	for _i in 0..steps {
		let temp = vec.pop().unwrap();
		vec.insert(0, temp);
	}
}
