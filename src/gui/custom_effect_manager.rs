use crate::gui::keyboard_color_tiles::KeyboardColorTiles;
use crate::keyboard_utils::{Keyboard, LightingEffects};
use fltk::{menu::Choice, prelude::*};
use image::buffer::ConvertBuffer;
use parking_lot::Mutex;
use rand::Rng;
use scrap::{Capturer, Display};
use std::{
	convert::TryInto,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	thread,
	time::{Duration, Instant},
};

const DISP_WIDTH: u32 = 2560;
const DISP_HEIGHT: u32 = 1600;

pub enum CustomEffects {
	Lightning,
	AmbientLight,
	SmoothLeftWave,
	SmoothRightWave,
	LeftSwipe,
	RightSwipe,
}

#[derive(Clone)]
pub struct CustomEffectManager {
	pub keyboard: Arc<Mutex<Keyboard>>,
	pub keyboard_color_tiles: KeyboardColorTiles,
	pub speed_choice: Choice,
	pub stop_signal: Arc<AtomicBool>,
	pub thread_ended_signal: Arc<AtomicBool>,
}

#[allow(dead_code)]
impl CustomEffectManager {
	pub fn change_effect(&mut self, effect: CustomEffects) {
		self.stop_signal.store(true, Ordering::Relaxed);
		while !self.thread_ended_signal.load(Ordering::Relaxed) {
			thread::sleep(Duration::from_millis(100));
		}
		self.stop_signal.store(false, Ordering::Relaxed);
		self.keyboard.lock().set_effect(LightingEffects::Static);

		match effect {
			CustomEffects::Lightning => {
				self.keyboard_color_tiles.deactivate();

				//Create necessary clones to be passed into thread
				let stop_signal = Arc::clone(&self.stop_signal);
				let keyboard = Arc::clone(&self.keyboard);
				let speed_choice = Arc::from(Mutex::from(self.speed_choice.clone()));
				let thread_ended_signal = Arc::clone(&self.thread_ended_signal);
				thread::spawn(move || {
					thread_ended_signal.store(false, Ordering::Relaxed);
					while !stop_signal.load(Ordering::Relaxed) {
						let zone = rand::thread_rng().gen_range(0..4);
						let steps = rand::thread_rng().gen_range(50..=200);
						keyboard.lock().set_zone_by_index(zone, [255.0; 3]);
						// keyboard.lock().set_colors_to(&[255.0; 12]);
						keyboard
							.lock()
							.transition_colors_to(&[0.0; 12], steps / speed_choice.lock().choice().unwrap().parse::<u8>().unwrap(), 5);
						let sleep_time = rand::thread_rng().gen_range(100..=2000);
						thread::sleep(Duration::from_millis(sleep_time));
					}
					thread_ended_signal.store(true, Ordering::Relaxed);
				});
			}
			CustomEffects::AmbientLight => {
				self.keyboard_color_tiles.deactivate();

				//Create necessary clones to be passed into thread
				let stop_signal = Arc::clone(&self.stop_signal);
				let keyboard = Arc::clone(&self.keyboard);
				let thread_ended_signal = Arc::clone(&self.thread_ended_signal);
				thread::spawn(move || {
					thread_ended_signal.store(false, Ordering::Relaxed);

					//Display setup
					let displays = Display::all().unwrap().len();
					for i in 0..displays {
						let display = Display::all().unwrap().remove(i);
						if display.width() == DISP_WIDTH as usize && display.height() == DISP_HEIGHT as usize {
							let mut capturer = Capturer::new(display, false).expect("Couldn't begin capture.");
							let (w, h) = (capturer.width(), capturer.height());

							let seconds_per_frame = Duration::from_nanos(1_000_000_000 / 30);
							type BgraImage<V> = image::ImageBuffer<image::Bgra<u8>, V>;
							while !stop_signal.load(Ordering::Relaxed) {
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
									keyboard.lock().transition_colors_to(&result, 4, 1);
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
							thread_ended_signal.store(true, Ordering::Relaxed);
							drop(capturer);
						}
					}
				});
			}
			CustomEffects::SmoothLeftWave => {
				self.keyboard_color_tiles.deactivate();

				//Create necessary clones to be passed into thread
				let stop_signal = Arc::clone(&self.stop_signal);
				let keyboard = Arc::clone(&self.keyboard);
				let speed_choice = Arc::from(Mutex::from(self.speed_choice.clone()));
				let thread_ended_signal = Arc::clone(&self.thread_ended_signal);

				thread::spawn(move || {
					let mut gradient = vec![255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 255.0, 255.0, 0.0, 255.0];
					thread_ended_signal.store(false, Ordering::Relaxed);
					while !stop_signal.load(Ordering::Relaxed) {
						shift_vec(&mut gradient, 3);
						let colors: [f32; 12] = gradient.clone().try_into().unwrap();
						keyboard.lock().transition_colors_to(&colors, 70 / speed_choice.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
						if stop_signal.load(Ordering::Relaxed) {
							break;
						}
						thread::sleep(Duration::from_millis(20));
					}
					thread_ended_signal.store(true, Ordering::Relaxed);
				});
			}
			CustomEffects::SmoothRightWave => {
				self.keyboard_color_tiles.deactivate();

				//Create necessary clones to be passed into thread
				let stop_signal = Arc::clone(&self.stop_signal);
				let keyboard = Arc::clone(&self.keyboard);
				let speed_choice = Arc::from(Mutex::from(self.speed_choice.clone()));
				let thread_ended_signal = Arc::clone(&self.thread_ended_signal);

				thread::spawn(move || {
					let mut gradient = vec![255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 255.0, 255.0, 0.0, 255.0];
					thread_ended_signal.store(false, Ordering::Relaxed);
					while !stop_signal.load(Ordering::Relaxed) {
						shift_vec(&mut gradient, 9);
						let colors: [f32; 12] = gradient.clone().try_into().unwrap();
						keyboard.lock().transition_colors_to(&colors, 70 / speed_choice.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
						if stop_signal.load(Ordering::Relaxed) {
							break;
						}
						thread::sleep(Duration::from_millis(20));
					}
					thread_ended_signal.store(true, Ordering::Relaxed);
				});
			}
			CustomEffects::LeftSwipe => {
				self.keyboard_color_tiles.activate();
				self.keyboard_color_tiles.master.deactivate();

				//Create necessary clones to be passed into thread
				let stop_signal = Arc::clone(&self.stop_signal);
				let keyboard = Arc::clone(&self.keyboard);
				let speed_choice = Arc::from(Mutex::from(self.speed_choice.clone()));
				let thread_ended_signal = Arc::clone(&self.thread_ended_signal);
				let keyboard_color_tiles = Arc::from(Mutex::from(self.keyboard_color_tiles.clone()));
				thread::spawn(move || {
					thread_ended_signal.store(false, Ordering::Relaxed);
					while !stop_signal.load(Ordering::Relaxed) {
						let zones_lock = &keyboard_color_tiles.lock().zones;
						let mut gradient = vec![
							zones_lock.left.red_input.value().parse::<f32>().unwrap(),
							zones_lock.left.green_input.value().parse::<f32>().unwrap(),
							zones_lock.left.blue_input.value().parse::<f32>().unwrap(),
							zones_lock.center_left.red_input.value().parse::<f32>().unwrap(),
							zones_lock.center_left.green_input.value().parse::<f32>().unwrap(),
							zones_lock.center_left.blue_input.value().parse::<f32>().unwrap(),
							zones_lock.center_right.red_input.value().parse::<f32>().unwrap(),
							zones_lock.center_right.green_input.value().parse::<f32>().unwrap(),
							zones_lock.center_right.blue_input.value().parse::<f32>().unwrap(),
							zones_lock.right.red_input.value().parse::<f32>().unwrap(),
							zones_lock.right.green_input.value().parse::<f32>().unwrap(),
							zones_lock.right.blue_input.value().parse::<f32>().unwrap(),
						];
						for _i in 0..4 {
							shift_vec(&mut gradient, 3);
							let colors: [f32; 12] = gradient.clone().try_into().unwrap();
							keyboard.lock().transition_colors_to(&colors, 150 / speed_choice.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							if stop_signal.load(Ordering::Relaxed) {
								break;
							}
						}
						if stop_signal.load(Ordering::Relaxed) {
							break;
						}
						thread::sleep(Duration::from_millis(20));
					}
					thread_ended_signal.store(true, Ordering::Relaxed);
				});
			}
			CustomEffects::RightSwipe => {
				self.keyboard_color_tiles.activate();
				self.keyboard_color_tiles.master.deactivate();

				//Create necessary clones to be passed into thread
				let stop_signal = Arc::clone(&self.stop_signal);
				let keyboard = Arc::clone(&self.keyboard);
				let speed_choice = Arc::from(Mutex::from(self.speed_choice.clone()));
				let thread_ended_signal = Arc::clone(&self.thread_ended_signal);
				let keyboard_color_tiles = Arc::from(Mutex::from(self.keyboard_color_tiles.clone()));
				thread::spawn(move || {
					thread_ended_signal.store(false, Ordering::Relaxed);
					while !stop_signal.load(Ordering::Relaxed) {
						let zones_lock = &keyboard_color_tiles.lock().zones;
						let mut gradient = vec![
							zones_lock.left.red_input.value().parse::<f32>().unwrap(),
							zones_lock.left.green_input.value().parse::<f32>().unwrap(),
							zones_lock.left.blue_input.value().parse::<f32>().unwrap(),
							zones_lock.center_left.red_input.value().parse::<f32>().unwrap(),
							zones_lock.center_left.green_input.value().parse::<f32>().unwrap(),
							zones_lock.center_left.blue_input.value().parse::<f32>().unwrap(),
							zones_lock.center_right.red_input.value().parse::<f32>().unwrap(),
							zones_lock.center_right.green_input.value().parse::<f32>().unwrap(),
							zones_lock.center_right.blue_input.value().parse::<f32>().unwrap(),
							zones_lock.right.red_input.value().parse::<f32>().unwrap(),
							zones_lock.right.green_input.value().parse::<f32>().unwrap(),
							zones_lock.right.blue_input.value().parse::<f32>().unwrap(),
						];
						for _i in 0..4 {
							shift_vec(&mut gradient, 9);
							let colors: [f32; 12] = gradient.clone().try_into().unwrap();
							keyboard.lock().transition_colors_to(&colors, 150 / speed_choice.lock().choice().unwrap().parse::<u8>().unwrap(), 10);
							if stop_signal.load(Ordering::Relaxed) {
								break;
							}
						}
						if stop_signal.load(Ordering::Relaxed) {
							break;
						}
						thread::sleep(Duration::from_millis(20));
					}
					thread_ended_signal.store(true, Ordering::Relaxed);
				});
			}
		}
	}
}

fn shift_vec(vec: &mut Vec<f32>, steps: u8) {
	for _i in 0..steps {
		let temp = vec.pop().unwrap();
		vec.insert(0, temp);
	}
}
