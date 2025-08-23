use std::{
    collections::HashMap,
    sync::{
        atomic::Ordering,
        mpsc::{Receiver, RecvTimeoutError},
    },
    thread,
    time::{Duration, Instant},
};

use fast_image_resize as fr;
use fr::{FilterType, ResizeAlg, Resizer};
use xcap::{Frame, Monitor, VideoRecorder, XCapResult};

use crate::{enums::MonitorId, manager::Inner};

pub struct Ambient {
    entries: HashMap<MonitorId, (VideoRecorder, Receiver<Frame>)>,
}

impl Ambient {
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    /// Get (or create) the persistent recorder+receiver for a monitor.
    pub fn get_or_create(&mut self, monitor_id: MonitorId) -> XCapResult<&mut (VideoRecorder, Receiver<Frame>)> {
        if !self.entries.contains_key(&monitor_id) {
            let monitor = get_monitor(monitor_id).expect(&format!("Failed to get monitor {}", monitor_id.0));
            let (recorder, rx) = monitor.video_recorder()?;

            self.entries.insert(monitor_id, (recorder, rx));
        } else {
        }

        Ok(self.entries.get_mut(&monitor_id).unwrap())
    }
}

pub fn get_monitor(monitor_id: MonitorId) -> Option<Monitor> {
    let mut monitors = Monitor::all().expect("Failed to enumerate monitors").into_iter();

    monitors
        .find(|m| m.id().as_ref().map(|id| *id == monitor_id.0).unwrap_or(false))
        .or_else(|| monitors.find(|m| m.is_primary().unwrap_or(false)))
        .or_else(|| monitors.next())
}

pub fn play(manager: &mut Inner, monitor_id: MonitorId, fps: u8, saturation_boost: f32) {
    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        // Select monitor (primary if not found)
        let (recorder, rx) = manager.ambient.get_or_create(monitor_id).expect("Couldn't get_or_create recorder");
        recorder.start().expect("Failed to start recording");

        let seconds_per_frame = Duration::from_nanos(1_000_000_000 / u64::from(fps));

        let mut resizer = Resizer::new();
        let resize_opts = fr::ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Box));

        // Destination 4x1 RGBA buffer reused every frame
        let mut dst_image = fr::images::Image::new(4, 1, fr::PixelType::U8x4);

        // Reused tiny RGBA & RGB buffers to avoid allocs in the hot path
        let mut rgba_small: [u8; 16] = [0; 16];
        let mut rgb_out: [u8; 12] = [0; 12];

        // Discover source size on first frame
        let mut src_dimension: Option<(u32, u32)> = None;

        while !manager.stop_signals.keyboard_stop_signal.load(Ordering::SeqCst) {
            let now = Instant::now();

            match rx.recv_timeout(seconds_per_frame) {
                Ok(mut frame) => {
                    // Drain the buffer, we only care about the last frame
                    while let Ok(new_frame) = rx.try_recv() {
                        frame = new_frame;
                    }

                    // Remember source dimensions after first frame
                    let (src_w, src_h) = *src_dimension.get_or_insert((frame.width, frame.height));

                    let src_ref = fr::images::ImageRef::new(src_w, src_h, &frame.raw, fr::PixelType::U8x4).expect("invalid src view");

                    // Resize into 4x1 RGBA image
                    resizer.resize(&src_ref, &mut dst_image, Some(&resize_opts)).expect("resize failed");

                    // Copy the 16 bytes into our scratch so Photon can work on it
                    rgba_small.copy_from_slice(dst_image.buffer());

                    // Saturation
                    let mut img = photon_rs::PhotonImage::new(rgba_small.to_vec(), 4, 1);
                    photon_rs::colour_spaces::saturate_hsv(&mut img, saturation_boost);
                    let raw = img.get_raw_pixels();

                    // RGBA to RGB
                    for (src, dst) in raw.chunks_exact(4).zip(rgb_out.chunks_exact_mut(3)) {
                        dst[0] = src[0];
                        dst[1] = src[1];
                        dst[2] = src[2];
                    }

                    manager.keyboard.set_colors_to(&rgb_out).unwrap();
                }
                Err(RecvTimeoutError::Timeout) => { /* Keep going */ }
                Err(_) => break,
            }

            let elapsed_time = now.elapsed();
            if elapsed_time < seconds_per_frame {
                thread::sleep(seconds_per_frame - elapsed_time);
            }
        }
    }
}
