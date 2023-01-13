use std::{
    num::NonZeroU32,
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

use fast_image_resize as fr;

use scrap::{Capturer, Display, TraitCapturer};

use crate::enums::Message;

pub(super) struct AmbientLight;

impl AmbientLight {
    pub fn play(manager: &mut super::Inner, fps: u8) {
        //Display setup
        let display = Display::all().unwrap().remove(0);

        let mut capturer = Capturer::new(display, false).expect("Couldn't begin capture.");

        let (width, height) = (NonZeroU32::new(capturer.width() as u32).unwrap(), NonZeroU32::new(capturer.height() as u32).unwrap());
        let (dst_width, dst_height) = (NonZeroU32::new(4).unwrap(), NonZeroU32::new(1).unwrap());

        let seconds_per_frame = Duration::from_nanos(1_000_000_000 / fps as u64);
        let wait_base = seconds_per_frame.as_millis();
        let mut wait = wait_base;
        let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Box));

        while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
            if manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
                break;
            }

            let now = Instant::now();
            match capturer.frame(Duration::from_millis(wait as u64)) {
                Ok(frame) => {
                    // Adapted from https://github.com/Cykooz/fast_image_resize#resize-image
                    // Read source image from file

                    // HACK: Override opacity manually to ensure some kind of output because of jank elsewhere
                    let mut frame_vec = frame.to_vec();
                    for rgba in frame_vec.chunks_exact_mut(4) {
                        rgba[3] = 255;
                    }

                    let mut src_image = fr::Image::from_vec_u8(width, height, frame_vec, fr::PixelType::U8x4).unwrap();

                    // Create MulDiv instance
                    let alpha_mul_div: fr::MulDiv = fr::MulDiv::default();
                    // Multiple RGB channels of source image by alpha channel
                    alpha_mul_div.multiply_alpha_inplace(&mut src_image.view_mut()).unwrap();

                    // Create container for data of destination image
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

                    manager.keyboard.set_colors_to(&rgb).unwrap();
                    let elapsed_time = now.elapsed();
                    if elapsed_time < seconds_per_frame {
                        thread::sleep(seconds_per_frame - elapsed_time);
                    }
                }
                Err(error) => match error.kind() {
                    std::io::ErrorKind::WouldBlock => {
                        wait = wait_base - now.elapsed().as_millis();
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
