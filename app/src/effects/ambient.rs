use std::{
    num::NonZeroU32,
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

use fast_image_resize as fr;

use fr::Resizer;
use scrap::{Capturer, Display, Frame, TraitCapturer};

#[derive(Clone, Copy)]
struct ScreenDimensions {
    src: (NonZeroU32, NonZeroU32),
    dest: (NonZeroU32, NonZeroU32),
}

pub(super) struct AmbientLight;

impl AmbientLight {
    pub fn play(manager: &mut super::Inner, fps: u8) {
        while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
            //Display setup
            let display = Display::all().unwrap().remove(0);

            let mut capturer = Capturer::new(display, false).expect("Couldn't begin capture.");

            let dimensions = ScreenDimensions {
                src: (NonZeroU32::new(capturer.width() as u32).unwrap(), NonZeroU32::new(capturer.height() as u32).unwrap()),
                dest: (NonZeroU32::new(4).unwrap(), NonZeroU32::new(1).unwrap()),
            };

            let seconds_per_frame = Duration::from_nanos(1_000_000_000 / fps as u64);
            let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Box));

            #[cfg(target_os = "windows")]
            let mut try_gdi = 1;

             while !manager.stop_signals.keyboard_stop_signal.load(Ordering::SeqCst) {
                let now = Instant::now();

                match capturer.frame(seconds_per_frame) {
                    Ok(frame) => {
                        let rgb = process_frame(frame, dimensions, &mut resizer);

                        manager.keyboard.set_colors_to(&rgb).unwrap();
                        #[cfg(target_os = "windows")]
                        {
                            try_gdi = 0;
                        }
                    }
                    Err(error) => match error.kind() {
                        std::io::ErrorKind::WouldBlock =>
                        {
                            #[cfg(target_os = "windows")]
                            if try_gdi > 0 && !capturer.is_gdi() {
                                if try_gdi > 3 {
                                    capturer.set_gdi();
                                    try_gdi = 0;
                                }
                                try_gdi += 1;
                            }
                        }
                        _ =>
                        {
                            #[cfg(windows)]
                            if !capturer.is_gdi() {
                                capturer.set_gdi();
                                continue;
                            }
                        }
                    },
                }

                let elapsed_time = now.elapsed();
                if elapsed_time < seconds_per_frame {
                    thread::sleep(seconds_per_frame - elapsed_time);
                }
            }
        }
    }
}

fn process_frame(frame: Frame, dimensions: ScreenDimensions, resizer: &mut Resizer) -> [u8; 12] {
    // Adapted from https://github.com/Cykooz/fast_image_resize#resize-image
    // Read source image from file

    // HACK: Override opacity manually to ensure some kind of output because of jank elsewhere
    let mut frame_vec = frame.to_vec();

    for rgba in frame_vec.chunks_exact_mut(4) {
        rgba[3] = 255;
    }

    let mut src_image = fr::Image::from_vec_u8(dimensions.src.0, dimensions.src.1, frame_vec, fr::PixelType::U8x4).unwrap();

    // Create MulDiv instance
    let alpha_mul_div: fr::MulDiv = fr::MulDiv::default();
    // Multiple RGB channels of source image by alpha channel
    alpha_mul_div.multiply_alpha_inplace(&mut src_image.view_mut()).unwrap();

    // Create container for data of destination image
    let mut dst_image = fr::Image::new(dimensions.dest.0, dimensions.dest.1, fr::PixelType::U8x4);

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

    rgb
}
