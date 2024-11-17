use std::{sync::atomic::Ordering, thread, time::Duration};

use rand::Rng;

pub fn play(manager: &mut super::Inner, thread_rng: &mut rand::rngs::ThreadRng) {
    let xmas_color_array = [[255, 10, 10], [255, 255, 20], [30, 255, 30], [70, 70, 255]];
    let subeffect_count = 4;
    let mut last_subeffect = -1;
    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let mut subeffect = thread_rng.gen_range(0..subeffect_count);
        while last_subeffect == subeffect {
            subeffect = thread_rng.gen_range(0..subeffect_count);
        }
        last_subeffect = subeffect;

        match subeffect {
            0 => {
                for _i in 0..3 {
                    for colors in xmas_color_array {
                        manager.keyboard.solid_set_colors_to(colors).unwrap();
                        thread::sleep(Duration::from_millis(500));
                    }
                }
            }
            1 => {
                let color_1_index = thread_rng.gen_range(0..4);
                let used_colors_1: [u8; 3] = xmas_color_array[color_1_index];

                let mut color_2_index = thread_rng.gen_range(0..4);
                while color_1_index == color_2_index {
                    color_2_index = thread_rng.gen_range(0..4);
                }
                let used_colors_2: [u8; 3] = xmas_color_array[color_2_index];

                for _i in 0..4 {
                    manager.keyboard.solid_set_colors_to(used_colors_1).unwrap();
                    thread::sleep(Duration::from_millis(400));
                    manager.keyboard.solid_set_colors_to(used_colors_2).unwrap();
                    thread::sleep(Duration::from_millis(400));
                }
            }
            2 => {
                let steps = 100;
                manager.keyboard.transition_colors_to(&[0; 12], steps, 1).unwrap();
                let mut used_colors_array: [u8; 12] = [0; 12];
                let left_or_right = thread_rng.gen_range(0..2);
                if left_or_right == 0 {
                    for color in xmas_color_array {
                        for j in (0..12).step_by(3) {
                            used_colors_array[j] = color[0];
                            used_colors_array[j + 1] = color[1];
                            used_colors_array[j + 2] = color[2];
                            manager.keyboard.transition_colors_to(&used_colors_array, steps, 1).unwrap();
                        }
                        for j in (0..12).step_by(3) {
                            used_colors_array[j] = 0;
                            used_colors_array[j + 1] = 0;
                            used_colors_array[j + 2] = 0;
                            manager.keyboard.transition_colors_to(&used_colors_array, steps, 1).unwrap();
                        }
                    }
                } else {
                    for i in 0..4 {
                        for j in (0..12).step_by(3) {
                            used_colors_array[11 - j] = xmas_color_array[3 - i][0];
                            used_colors_array[11 - (j + 1)] = xmas_color_array[3 - i][1];
                            used_colors_array[11 - (j + 2)] = xmas_color_array[3 - i][2];
                            manager.keyboard.transition_colors_to(&used_colors_array, steps, 1).unwrap();
                        }
                        for j in (0..12).step_by(3) {
                            used_colors_array[11 - j] = 0;
                            used_colors_array[11 - (j + 1)] = 0;
                            used_colors_array[11 - (j + 2)] = 0;
                            manager.keyboard.transition_colors_to(&used_colors_array, steps, 1).unwrap();
                        }
                    }
                }
            }
            3 => {
                let state1 = [255, 255, 255, 0, 0, 0, 255, 255, 255, 0, 0, 0];
                let state2 = [0, 0, 0, 255, 255, 255, 0, 0, 0, 255, 255, 255];
                let steps = 30;
                for _i in 0..4 {
                    manager.keyboard.transition_colors_to(&state1, steps, 1).unwrap();
                    thread::sleep(Duration::from_millis(400));
                    manager.keyboard.transition_colors_to(&state2, steps, 1).unwrap();
                    thread::sleep(Duration::from_millis(400));
                }
            }
            _ => unreachable!("Subeffect index for Christmas effect is out of range."),
        }
    }
}
