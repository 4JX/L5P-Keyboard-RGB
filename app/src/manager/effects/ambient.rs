use std::{
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

use scrap::{Capturer, Display};

use crate::manager::Inner;

#[allow(dead_code)]
#[derive(Clone, Copy)]
struct ScreenDimensions {
    src: (u32, u32),
    dest: (u32, u32),
}

pub fn play(manager: &mut Inner, fps: u8, saturation_boost: f32, smoothness: bool) {
    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        //Display setup
        let display = match Display::all() {
            Ok(mut displays) if !displays.is_empty() => displays.remove(0),
            _ => {
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };

        let mut capturer = match Capturer::new(display) {
            Ok(c) => c,
            Err(_) => {
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };

        let dimensions = ScreenDimensions {
            src: (capturer.width() as u32, capturer.height() as u32),
            dest: (4, 1),
        };

        let seconds_per_frame = Duration::from_nanos(1_000_000_000 / u64::from(fps));

        #[cfg(target_os = "windows")]
        let mut _try_gdi = 0;

        let mut current_colors = [0f32; 12];
        let mut first_frame = true;

        while !manager.stop_signals.keyboard_stop_signal.load(Ordering::SeqCst) {
            let now = Instant::now();

            #[allow(clippy::single_match)]
            match capturer.frame() {
                Ok(frame) => {
                    let rgb = process_frame(&frame, dimensions, saturation_boost);
                    
                    if first_frame || !smoothness {
                        for i in 0..12 {
                            current_colors[i] = rgb[i] as f32;
                        }
                        first_frame = false;
                    } else {
                        // Exponential moving average for smooth color transitions
                        let alpha = 0.3; // Smoothing factor for buttery smooth transitions
                        for i in 0..12 {
                            current_colors[i] = current_colors[i] + alpha * (rgb[i] as f32 - current_colors[i]);
                        }
                    }
                    
                    let mut final_rgb = [0u8; 12];
                    for i in 0..12 {
                        final_rgb[i] = current_colors[i].round() as u8;
                    }

                    manager.keyboard.set_colors_to(&final_rgb).unwrap();
                }
                Err(error) => match error.kind() {
                    std::io::ErrorKind::WouldBlock => {
                        // DXGI timed out because the screen didn't change. Do nothing, this is normal.
                    }
                    _ => {
                        // Capturer failed (e.g. UAC prompt, resolution change, display sleep). 
                        // Break inner loop to re-initialize!
                        break;
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

fn process_frame(frame_vec: &[u8], dimensions: ScreenDimensions, saturation_boost: f32) -> [u8; 12] {
    let width = dimensions.src.0 as usize;
    let height = dimensions.src.1 as usize;
    let slice_width = width / 4;
    
    // Step size for downsampling. 8 provides a massive speedup (64x less pixels to read)
    // while maintaining a very accurate average color of the screen zones.
    let step = 8;
    
    let mut rgba = [0u8; 16];
    
    for zone in 0..4 {
        let start_x = zone * slice_width;
        let end_x = start_x + slice_width;
        
        let mut sum_r = 0u64;
        let mut sum_g = 0u64;
        let mut sum_b = 0u64;
        let mut count = 0u64;
        
        // Compute average by reading straight from the DXGI mapped buffer
        for y in (0..height).step_by(step) {
            let row_offset = y * width;
            for x in (start_x..end_x).step_by(step) {
                let idx = (row_offset + x) * 4;
                if idx + 2 < frame_vec.len() {
                    let b = frame_vec[idx];
                    let g = frame_vec[idx + 1];
                    let r = frame_vec[idx + 2];
                    
                    // ACCURATE averaging! Include pitch-black pixels to drop the average when screen is dark!
                    sum_r += r as u64;
                    sum_g += g as u64;
                    sum_b += b as u64;
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            rgba[zone * 4] = (sum_r / count) as u8;
            rgba[zone * 4 + 1] = (sum_g / count) as u8;
            rgba[zone * 4 + 2] = (sum_b / count) as u8;
        } else {
            rgba[zone * 4] = 0;
            rgba[zone * 4 + 1] = 0;
            rgba[zone * 4 + 2] = 0;
        }
        rgba[zone * 4 + 3] = 255;
    }

    let mut img = photon_rs::PhotonImage::new(rgba.to_vec(), 4, 1);
    photon_rs::colour_spaces::saturate_hsv(&mut img, saturation_boost);

    let raw = img.get_raw_pixels();
    let mut rgb: [u8; 12] = [0; 12];
    for (src, dst) in raw.chunks_exact(4).zip(rgb.chunks_exact_mut(3)) {
        dst[0] = src[0];
        dst[1] = src[1];
        dst[2] = src[2];
    }

    rgb
}
