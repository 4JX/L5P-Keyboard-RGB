use std::{
	num::NonZeroU32,
	sync::atomic::Ordering,
	thread,
	time::{Duration, Instant},
};

use fast_image_resize as fr;
use rand::rngs::ThreadRng;
use scrap::{Capturer, Display};

use crate::{enums::Message, profile::Profile};

use super::{EffectManager, EffectPlayer};

const DEFAULT_FPS: u8 = 12;

pub(super) struct AmbientLightWarmerDesaturated;

impl EffectPlayer for AmbientLightWarmerDesaturated {
	fn play(manager: &mut EffectManager, p: Profile, _thread_rng: &mut ThreadRng) {
		//Display setup
		let display = Display::all().unwrap().remove(0);

		let mut capturer = Capturer::new(display, false).expect("Couldn't begin capture.");
		let (w, h) = (capturer.width(), capturer.height());

		let fps = match p.effect {
			crate::enums::Effects::AmbientLightWarmerDesaturated
		 { fps } => {
				if fps < 1 {
					DEFAULT_FPS
				} else {
					fps
				}
			}
			_ => unreachable!("Attempted to play AmbientLightWarmerDesaturated
		 effect with wrong profile"),
		};

		let seconds_per_frame = Duration::from_nanos(1_000_000_000 / fps as u64);
		let wait_base: i32 = seconds_per_frame.as_millis() as i32;
		let mut wait = wait_base;
		let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3));

		while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
			if manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
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
						dst[0] = src[2] / 20 * 10;
						dst[1] = src[1] / 30 * 10;
						dst[2] = src[0] / 70 * 10;

						// * If too dim:
						if dst[0] < 31 && dst[1] < 15 && dst[2] < 9 {
							dst[0] = 32;
							dst[1] = 16;
							dst[2] = 10;
						} else {
							if dst[0] > dst[1] + dst[2] { // * if too red:
								dst[0] = src[2] / 3;
								dst[1] = src[2] / 7;
								dst[2] = src[2] / 7;
							}
							if dst[1] > dst[0] + dst[2] { // * if too green:
								dst[0] = src[1] / 3;
								dst[1] = src[1] / 2;
								dst[2] = src[1] / 3;
							}
							if dst[2] > dst[0] + dst[1] { // * if too blue:
								dst[0] = src[0] / 7;
								dst[1] = src[0] / 7;
								dst[2] = src[0] / 3;
							}
						}
					}

					manager.keyboard.set_colors_to(&rgb);
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
						manager.stop_signals.store_true();
						manager.tx.send(Message::Refresh).unwrap();
					}

					_ => {}
				},
			}
		}
	}
}
