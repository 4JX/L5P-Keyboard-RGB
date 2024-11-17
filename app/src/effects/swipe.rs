use std::{sync::atomic::Ordering, thread, time::Duration};

use crate::{enums::Direction, profile::Profile};

pub fn play(manager: &mut super::Inner, p: &Profile) {
    let mut rgb_array = p.rgb_array();

    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        if manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
            break;
        }

        match p.direction {
            Direction::Left => rgb_array.rotate_right(3),
            Direction::Right => rgb_array.rotate_left(3),
        }

        manager.keyboard.transition_colors_to(&rgb_array, 150 / p.speed, 10).unwrap();

        if manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
}
