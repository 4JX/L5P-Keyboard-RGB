use std::{sync::atomic::Ordering, thread, time::Duration};

use crate::{
    enums::{Direction, SwipeMode},
    manager::{profile::Profile, Inner},
};

const STEPS: u8 = 150;

pub fn play(manager: &mut Inner, profile: &Profile, mode: SwipeMode, clean_with_black: bool) {
    let mut change_rgb_array = profile.rgb_array();
    let fill_rgb_array = profile.rgb_array();
    let steps = STEPS / profile.speed;

    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        match mode {
            SwipeMode::Change => {
                match profile.direction {
                    Direction::Left => change_rgb_array.rotate_right(3),
                    Direction::Right => change_rgb_array.rotate_left(3),
                }
                manager.keyboard.transition_colors_to(&change_rgb_array, steps, 10).unwrap();
            }
            SwipeMode::Fill => {
                let mut used_colors_array: [u8; 12] = [0; 12];

                // A little hacky to avoid type mismatch errors, but you gotta do what you gotta do
                let range: Vec<usize> = match profile.direction {
                    Direction::Left => (0..4).collect(),
                    Direction::Right => (0..4).rev().collect(),
                };

                for i in range.clone() {
                    for j in range.clone() {
                        used_colors_array[j * 3] = fill_rgb_array[i * 3];
                        used_colors_array[j * 3 + 1] = fill_rgb_array[i * 3 + 1];
                        used_colors_array[j * 3 + 2] = fill_rgb_array[i * 3 + 2];
                        manager.keyboard.transition_colors_to(&used_colors_array, steps, 1).unwrap();
                    }
                    if clean_with_black {
                        for j in range.clone() {
                            used_colors_array[j * 3] = 0;
                            used_colors_array[j * 3 + 1] = 0;
                            used_colors_array[j * 3 + 2] = 0;
                            manager.keyboard.transition_colors_to(&used_colors_array, steps, 1).unwrap();
                        }
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(20));
    }
}
