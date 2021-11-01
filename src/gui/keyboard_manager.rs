use super::enums::Message;
use super::{enums::Effects, keyboard_color_tiles::KeyboardColorTiles};
use crate::keyboard_utils::{BaseEffects, Keyboard};
use fltk::app;
use fltk::{menu::Choice, prelude::*};
use image::buffer::ConvertBuffer;
use rand::Rng;
use scrap::{Capturer, Display};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::{
	convert::TryInto,
	sync::{atomic::Ordering, Arc},
	thread,
	time::{Duration, Instant},
};

const DISP_WIDTH: u32 = 2560;
const DISP_HEIGHT: u32 = 1600;

pub struct KeyboardManager {
	pub keyboard: Keyboard,
	pub rx: Receiver<Message>,
}

impl KeyboardManager {
	pub fn start(&mut self, mut keyboard_color_tiles: &mut KeyboardColorTiles, mut speed_choice: &mut Choice, stop_signal: &Arc<AtomicBool>) {
		loop {
			if let Ok(message) = self.rx.recv() {
				match message {
					Message::UpdateEffect { effect } => {
						stop_signal.store(true, Ordering::Relaxed);
						while !stop_signal.load(Ordering::Relaxed) {
							thread::sleep(Duration::from_millis(100));
						}
						self.change_effect(effect, &mut keyboard_color_tiles, &mut speed_choice, stop_signal);
					}
					Message::UpdateAllValues { value } => {
						self.keyboard.set_colors_to(&value);
					}
					Message::UpdateRGB { index, value } => {
						self.keyboard.solid_set_value_by_index(index, value);
					}
					Message::UpdateZone { zone_index, value } => {
						self.keyboard.set_zone_by_index(zone_index, value);
					}
					Message::UpdateValue { index, value } => {
						self.keyboard.set_value_by_index(index, value);
					}
					Message::UpdateBrightness { brightness: value } => {
						self.keyboard.set_brightness(value);
					}
					Message::UpdateSpeed { speed: value } => {
						self.keyboard.set_speed(value);
					}
				}
				app::awake();
			}
		}
	}
	pub fn change_effect(&mut self, effect: Effects, keyboard_color_tiles: &mut KeyboardColorTiles, speed_choice: &mut Choice, stop_signal: &Arc<AtomicBool>) {
		stop_signal.store(false, Ordering::Relaxed);
		self.keyboard.set_effect(BaseEffects::Static);

		match effect {
			Effects::Static => {
				let values = keyboard_color_tiles.zones.get_values();
				self.keyboard.set_colors_to(&values);
				self.keyboard.set_effect(BaseEffects::Static);
				keyboard_color_tiles.activate();
			}
			Effects::Breath => {
				let values = keyboard_color_tiles.zones.get_values();
				self.keyboard.set_colors_to(&values);
				self.keyboard.set_effect(BaseEffects::Breath);
				keyboard_color_tiles.activate();
			}
			Effects::Smooth => {
				keyboard_color_tiles.deactivate();
				self.keyboard.set_effect(BaseEffects::Smooth);
			}
			Effects::LeftWave => {
				keyboard_color_tiles.deactivate();
				self.keyboard.set_effect(BaseEffects::LeftWave);
			}
			Effects::RightWave => {
				keyboard_color_tiles.deactivate();
				self.keyboard.set_effect(BaseEffects::RightWave);
			}
			Effects::Lightning => {
				keyboard_color_tiles.deactivate();
				app::awake();

				while !stop_signal.load(Ordering::Relaxed) {
					if stop_signal.load(Ordering::Relaxed) {
						break;
					}
					let zone = rand::thread_rng().gen_range(0..4);
					let steps = rand::thread_rng().gen_range(50..=200);
					self.keyboard.set_zone_by_index(zone, [255.0; 3]);
					self.keyboard.transition_colors_to(&[0.0; 12], steps / speed_choice.choice().unwrap().parse::<u8>().unwrap(), 5);
					let sleep_time = rand::thread_rng().gen_range(100..=2000);
					thread::sleep(Duration::from_millis(sleep_time));
				}
			}
			Effects::AmbientLight => {
				keyboard_color_tiles.deactivate();
				app::awake();

				//Display setup
				let displays = Display::all().unwrap().len();
				for i in 0..displays {
					let display = Display::all().unwrap().remove(i);
					if display.width() == DISP_WIDTH as usize && display.height() == DISP_HEIGHT as usize {
						type BgraImage<V> = image::ImageBuffer<image::Bgra<u8>, V>;
						let mut capturer = Capturer::new(display, false).expect("Couldn't begin capture.");
						let (w, h) = (capturer.width(), capturer.height());

						let seconds_per_frame = Duration::from_nanos(1_000_000_000 / 30);
						while !stop_signal.load(Ordering::Relaxed) {
							if stop_signal.load(Ordering::Relaxed) {
								break;
							}
							if let Ok(frame) = capturer.frame(0) {
								let now = Instant::now();
								let bgra_img = BgraImage::from_raw(w as u32, h as u32, &*frame).expect("Could not get bgra image.");
								let rgb_img: image::RgbImage = bgra_img.convert();
								let resized = image::imageops::resize(&rgb_img, 4, 1, image::imageops::FilterType::Lanczos3);
								let dst = resized.into_vec();

								let mut result: [f32; 12] = [0.0; 12];
								for i in 0..12 {
									result[i] = dst[i] as f32;
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
									if display.width() == DISP_WIDTH as usize && display.height() == DISP_HEIGHT as usize {
										capturer = Capturer::new(display, false).expect("Couldn't begin capture.");
									}
								}
							}
							thread::sleep(Duration::from_millis(20));
						}
						drop(capturer);
					}
				}
			}
			Effects::SmoothLeftWave => {
				keyboard_color_tiles.deactivate();
				app::awake();

				let mut gradient = vec![255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 255.0, 255.0, 0.0, 255.0];

				while !stop_signal.load(Ordering::Relaxed) {
					if stop_signal.load(Ordering::Relaxed) {
						break;
					}
					shift_vec(&mut gradient, 3);
					let colors: [f32; 12] = gradient.clone().try_into().unwrap();
					self.keyboard.transition_colors_to(&colors, 70 / speed_choice.choice().unwrap().parse::<u8>().unwrap(), 10);
					if stop_signal.load(Ordering::Relaxed) {
						break;
					}
					thread::sleep(Duration::from_millis(20));
				}
			}
			Effects::SmoothRightWave => {
				keyboard_color_tiles.deactivate();
				app::awake();

				let mut gradient = vec![255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 255.0, 255.0, 0.0, 255.0];

				while !stop_signal.load(Ordering::Relaxed) {
					if stop_signal.load(Ordering::Relaxed) {
						break;
					}
					shift_vec(&mut gradient, 9);
					let colors: [f32; 12] = gradient.clone().try_into().unwrap();
					self.keyboard.transition_colors_to(&colors, 70 / speed_choice.choice().unwrap().parse::<u8>().unwrap(), 10);
					if stop_signal.load(Ordering::Relaxed) {
						break;
					}
					thread::sleep(Duration::from_millis(20));
				}
			}
			Effects::LeftSwipe => {
				keyboard_color_tiles.activate();
				keyboard_color_tiles.master.deactivate();
				app::awake();

				while !stop_signal.load(Ordering::Relaxed) {
					if stop_signal.load(Ordering::Relaxed) {
						break;
					}

					let mut gradient = keyboard_color_tiles.zones.get_values().to_vec();
					for _i in 0..4 {
						shift_vec(&mut gradient, 3);
						let colors: [f32; 12] = gradient.clone().try_into().unwrap();
						self.keyboard.transition_colors_to(&colors, 150 / speed_choice.choice().unwrap().parse::<u8>().unwrap(), 10);
						if stop_signal.load(Ordering::Relaxed) {
							break;
						}
					}
					if stop_signal.load(Ordering::Relaxed) {
						break;
					}
					thread::sleep(Duration::from_millis(20));
				}
			}
			Effects::RightSwipe => {
				keyboard_color_tiles.activate();
				keyboard_color_tiles.master.deactivate();
				app::awake();

				while !stop_signal.load(Ordering::Relaxed) {
					if stop_signal.load(Ordering::Relaxed) {
						break;
					}

					let mut gradient = keyboard_color_tiles.zones.get_values().to_vec();
					for _i in 0..4 {
						shift_vec(&mut gradient, 9);
						let colors: [f32; 12] = gradient.clone().try_into().unwrap();
						self.keyboard.transition_colors_to(&colors, 150 / speed_choice.choice().unwrap().parse::<u8>().unwrap(), 10);
						if stop_signal.load(Ordering::Relaxed) {
							break;
						}
					}
					if stop_signal.load(Ordering::Relaxed) {
						break;
					}
					thread::sleep(Duration::from_millis(20));
				}
			}
		}
		stop_signal.store(false, Ordering::Relaxed);
	}
}

fn shift_vec(vec: &mut Vec<f32>, steps: u8) {
	for _i in 0..steps {
		let temp = vec.pop().unwrap();
		vec.insert(0, temp);
	}
}
