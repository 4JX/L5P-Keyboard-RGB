use std::{sync::atomic::Ordering, thread, time::Duration};

use sysinfo::{Components, System};

use crate::manager::Inner;

pub fn play(manager: &mut Inner) {
    let safe_temp = 20.0;
    let ramp_boost = 1.6;
    let temp_cool: [f32; 12] = [0.0, 255.0, 0.0, 0.0, 255.0, 0.0, 0.0, 255.0, 0.0, 0.0, 255.0, 0.0];
    let temp_hot: [f32; 12] = [255.0, 0.0, 0.0, 255.0, 0.0, 0.0, 255.0, 0.0, 0.0, 255.0, 0.0, 0.0];

    let mut color_differences: [f32; 12] = [0.0; 12];
    for index in 0..12 {
        color_differences[index] = temp_hot[index] - temp_cool[index];
    }

    let mut sys = System::new_all();
    sys.refresh_all();

    let mut components = Components::new_with_refreshed_list();

    for component in components.iter_mut() {
        if component.label().contains("Tctl") {
            while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
                component.refresh();
                let mut adjusted_temp = component.temperature() - safe_temp;
                if adjusted_temp < 0.0 {
                    adjusted_temp = 0.0;
                }
                let temp_percent = (adjusted_temp / 100.0) * ramp_boost;

                let mut target = [0.0; 12];
                for index in 0..12 {
                    target[index] = color_differences[index].mul_add(temp_percent, temp_cool[index]);
                }
                manager.keyboard.transition_colors_to(&target.map(|val| val as u8), 5, 1).unwrap();
                thread::sleep(Duration::from_millis(200));
            }
        }
    }
}
