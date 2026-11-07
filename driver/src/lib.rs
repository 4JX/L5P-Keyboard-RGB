use error::{RangeError, RangeErrorKind, Result};
use hidapi::{HidApi, HidDevice};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

pub mod error;

const KNOWN_DEVICE_INFOS: [(u16, u16, u16, u16); 12] = [
    (0x048d, 0xc995, 0xff89, 0x00cc), // 2024 Pro
    (0x048d, 0xc994, 0xff89, 0x00cc), // 2024
    (0x048d, 0xc993, 0xff89, 0x00cc), // 2024 LOQ
    (0x048d, 0xc985, 0xff89, 0x00cc), // 2023 Pro
    (0x048d, 0xc984, 0xff89, 0x00cc), // 2023
    (0x048d, 0xc983, 0xff89, 0x00cc), // 2023 LOQ
    (0x048d, 0xc975, 0xff89, 0x00cc), // 2022
    (0x048d, 0xc973, 0xff89, 0x00cc), // 2022 Ideapad
    (0x048d, 0xc965, 0xff89, 0x00cc), // 2021
    (0x048d, 0xc963, 0xff89, 0x00cc), // 2021 Ideapad
    (0x048d, 0xc955, 0xff89, 0x00cc), // 2020
    (0x048d, 0xc693, 0xff89, 0x00cc), // 2025 LOQ 15IRX10 (fallback)
];

// Lenovo LOQ 15IRX10 (2025, model 83JE) exposes its keyboard through the
// standard HID Lighting and Illumination usage page instead of the vendor
// collection used by older Legion/LOQ models.
const LOQ_15IRX10_VID: u16 = 0x048d;
const LOQ_15IRX10_PID: u16 = 0xc693;
const LAMP_ARRAY_USAGE_PAGE: u16 = 0x0059;
const LAMP_ARRAY_USAGE: u16 = 0x0001;

const LAMP_ARRAY_ATTRIBUTES_REPORT: u8 = 1;
const LAMP_ARRAY_MULTI_UPDATE_REPORT: u8 = 4;
const LAMP_ARRAY_RANGE_UPDATE_REPORT: u8 = 5;
const LAMP_ARRAY_CONTROL_REPORT: u8 = 6;
const LAMP_ARRAY_FEATURE_LEN: usize = 51;
const LOQ_15IRX10_LAMP_COUNT: usize = 24;

pub const SPEED_RANGE: std::ops::RangeInclusive<u8> = 1..=4;
pub const BRIGHTNESS_RANGE: std::ops::RangeInclusive<u8> = 1..=2;
pub const ZONE_RANGE: std::ops::RangeInclusive<u8> = 0..=3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaseEffects {
    Static,
    Breath,
    Smooth,
    LeftWave,
    RightWave,
}

#[derive(Clone, Copy)]
pub struct LightingState {
    effect_type: BaseEffects,
    speed: u8,
    brightness: u8,
    rgb_values: [u8; 12],
}

enum Protocol {
    Legacy,
    LampArray,
}

struct LampArrayHandle {
    state: Arc<Mutex<LightingState>>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

pub struct Keyboard {
    keyboard_hid: Option<HidDevice>,
    current_state: LightingState,
    stop_signal: Arc<AtomicBool>,
    protocol: Protocol,
    lamp: Option<LampArrayHandle>,
    // Only kept alive so its Drop restores the Windows lighting registry
    // values when the keyboard is released.
    #[allow(dead_code)]
    #[cfg(target_os = "windows")]
    lighting_restore: Option<LightingRestore>,
}

#[allow(dead_code)]
impl Keyboard {
    fn build_payload(&self) -> Result<[u8; 33]> {
        let keyboard_state = &self.current_state;

        if !SPEED_RANGE.contains(&keyboard_state.speed) {
            return Err(RangeError { kind: RangeErrorKind::Speed }.into());
        }
        if !BRIGHTNESS_RANGE.contains(&keyboard_state.brightness) {
            return Err(RangeError { kind: RangeErrorKind::Brightness }.into());
        }

        let mut payload: [u8; 33] = [0; 33];
        payload[0] = 0xcc;
        payload[1] = 0x16;
        payload[2] = match keyboard_state.effect_type {
            BaseEffects::Static => 0x01,
            BaseEffects::Breath => 0x03,
            BaseEffects::Smooth => 0x06,
            BaseEffects::LeftWave => {
                payload[19] = 0x1;
                0x04
            }
            BaseEffects::RightWave => {
                payload[18] = 0x1;
                0x04
            }
        };

        payload[3] = keyboard_state.speed;
        payload[4] = keyboard_state.brightness;

        if let BaseEffects::Static | BaseEffects::Breath = keyboard_state.effect_type {
            payload[5..(12 + 5)].copy_from_slice(&keyboard_state.rgb_values[..12]);
        };

        Ok(payload)
    }

    fn sync_lamp_state(&self) {
        if let Some(lamp) = &self.lamp {
            if let Ok(mut state) = lamp.state.lock() {
                *state = self.current_state;
            }
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        match self.protocol {
            Protocol::Legacy => {
                let payload = self.build_payload()?;
                let device = self.keyboard_hid.as_ref().ok_or(error::Error::DeviceNotFound)?;
                device.send_feature_report(&payload)?;
            }
            Protocol::LampArray => self.sync_lamp_state(),
        }

        Ok(())
    }

    pub fn set_effect(&mut self, effect: BaseEffects) -> Result<()> {
        self.current_state.effect_type = effect;
        self.refresh()?;

        Ok(())
    }

    pub fn set_speed(&mut self, speed: u8) -> Result<()> {
        if !SPEED_RANGE.contains(&speed) {
            return Err(RangeError { kind: RangeErrorKind::Speed }.into());
        }

        self.current_state.speed = speed;
        self.refresh()?;

        Ok(())
    }

    pub fn set_brightness(&mut self, brightness: u8) -> Result<()> {
        if !BRIGHTNESS_RANGE.contains(&brightness) {
            return Err(RangeError { kind: RangeErrorKind::Brightness }.into());
        }
        let brightness = brightness.clamp(BRIGHTNESS_RANGE.min().unwrap(), BRIGHTNESS_RANGE.max().unwrap());
        self.current_state.brightness = brightness;
        self.refresh()?;

        Ok(())
    }

    pub fn set_zone_by_index(&mut self, zone_index: u8, new_values: [u8; 3]) -> Result<()> {
        if !ZONE_RANGE.contains(&zone_index) {
            return Err(RangeError { kind: RangeErrorKind::Zone }.into());
        }

        for (i, _) in new_values.iter().enumerate() {
            let full_index = (zone_index * 3 + i as u8) as usize;
            self.current_state.rgb_values[full_index] = new_values[i];
        }
        self.refresh()?;

        Ok(())
    }

    pub fn set_colors_to(&mut self, new_values: &[u8; 12]) -> Result<()> {
        // Keep the colors even while a dynamic effect is running so switching
        // back to a static effect uses the latest selection.
        for (i, _) in new_values.iter().enumerate() {
            self.current_state.rgb_values[i] = new_values[i];
        }

        if let BaseEffects::Static | BaseEffects::Breath = self.current_state.effect_type {
            self.refresh()?;
        }

        Ok(())
    }

    pub fn solid_set_colors_to(&mut self, new_values: [u8; 3]) -> Result<()> {
        for i in (0..12).step_by(3) {
            self.current_state.rgb_values[i] = new_values[0];
            self.current_state.rgb_values[i + 1] = new_values[1];
            self.current_state.rgb_values[i + 2] = new_values[2];
        }

        if let BaseEffects::Static | BaseEffects::Breath = self.current_state.effect_type {
            self.refresh()?;
        }

        Ok(())
    }

    pub fn transition_colors_to(&mut self, target_colors: &[u8; 12], steps: u8, delay_between_steps: u64) -> Result<()> {
        if let BaseEffects::Static | BaseEffects::Breath = self.current_state.effect_type {
            let mut new_values = self.current_state.rgb_values.map(f32::from);
            let mut color_differences: [f32; 12] = [0.0; 12];
            for index in 0..12 {
                color_differences[index] = (f32::from(target_colors[index]) - f32::from(self.current_state.rgb_values[index])) / f32::from(steps);
            }
            if !self.stop_signal.load(Ordering::SeqCst) {
                for _step_num in 1..=steps {
                    if self.stop_signal.load(Ordering::SeqCst) {
                        break;
                    }
                    for (index, _) in color_differences.iter().enumerate() {
                        new_values[index] += color_differences[index];
                    }
                    self.current_state.rgb_values = new_values.map(|val| val as u8);

                    self.refresh()?;
                    thread::sleep(Duration::from_millis(delay_between_steps));
                }
                self.set_colors_to(target_colors)?;
            }
        }

        Ok(())
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        if let Some(mut lamp) = self.lamp.take() {
            lamp.stop.store(true, Ordering::SeqCst);
            if let Some(handle) = lamp.thread.take() {
                let _ = handle.join();
            }
        }

        // `lighting_restore` restores the Windows Dynamic Lighting values when
        // it is dropped after this method returns.
    }
}

fn is_loq_15irx10(info: &hidapi::DeviceInfo) -> bool {
    info.vendor_id() == LOQ_15IRX10_VID
        && info.product_id() == LOQ_15IRX10_PID
        && info.usage_page() == LAMP_ARRAY_USAGE_PAGE
        && info.usage() == LAMP_ARRAY_USAGE
}

fn expand_zones_to_lamps(rgb: &[u8; 12], lamp_count: usize) -> Vec<[u8; 3]> {
    let lamp_count = lamp_count.max(1);
    let zones = 4.min(lamp_count);
    (0..lamp_count)
        .map(|i| {
            let zone = (i * zones / lamp_count).min(zones.saturating_sub(1));
            [rgb[zone * 3], rgb[zone * 3 + 1], rgb[zone * 3 + 2]]
        })
        .collect()
}

fn scale_rgb(color: [u8; 3], factor: f32) -> [u8; 3] {
    [
        (f32::from(color[0]) * factor).round().clamp(0.0, 255.0) as u8,
        (f32::from(color[1]) * factor).round().clamp(0.0, 255.0) as u8,
        (f32::from(color[2]) * factor).round().clamp(0.0, 255.0) as u8,
    ]
}

fn hsv_to_rgb(hue: f64, saturation: f64, value: f64) -> [u8; 3] {
    let h = hue.rem_euclid(360.0) / 60.0;
    let c = value * saturation;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = value - c;
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    ]
}

fn compute_lamps(state: &LightingState, lamp_count: usize, elapsed: f64) -> Vec<[u8; 3]> {
    let zones = expand_zones_to_lamps(&state.rgb_values, lamp_count);
    let speed = f64::from(state.speed.clamp(1, 4));

    match state.effect_type {
        BaseEffects::Static => zones,
        BaseEffects::Breath => {
            let factor = ((elapsed * speed * 1.6).sin() * 0.5 + 0.5) as f32;
            zones.iter().map(|color| scale_rgb(*color, factor)).collect()
        }
        BaseEffects::Smooth => {
            let hue = (elapsed * speed * 60.0) % 360.0;
            (0..lamp_count)
                .map(|i| {
                    let t = if lamp_count <= 1 { 0.0 } else { i as f64 / (lamp_count - 1) as f64 };
                    hsv_to_rgb(hue + t * 90.0, 1.0, 1.0)
                })
                .collect()
        }
        BaseEffects::LeftWave | BaseEffects::RightWave => {
            let direction = if matches!(state.effect_type, BaseEffects::LeftWave) { 1.0 } else { -1.0 };
            let hue = (elapsed * speed * 120.0) % 360.0;
            (0..lamp_count)
                .map(|i| {
                    let t = i as f64 / lamp_count as f64;
                    hsv_to_rgb((hue + t * 360.0 * direction).rem_euclid(360.0), 1.0, 1.0)
                })
                .collect()
        }
    }
}

fn count_color_runs(lamps: &[[u8; 3]]) -> usize {
    let mut runs = 0;
    let mut i = 0;
    while i < lamps.len() {
        let color = lamps[i];
        i += 1;
        runs += 1;
        while i < lamps.len() && lamps[i] == color {
            i += 1;
        }
    }
    runs
}

fn send_lamp_array_control(device: &HidDevice, autonomous_mode: u8) {
    let mut buffer = [0u8; LAMP_ARRAY_FEATURE_LEN];
    buffer[0] = LAMP_ARRAY_CONTROL_REPORT;
    buffer[1] = autonomous_mode;
    let _ = device.send_feature_report(&buffer);
}

fn send_lamps(device: &HidDevice, lamps: &[[u8; 3]], intensity: u8) -> usize {
    let lamp_count = lamps.len();
    if lamp_count == 0 {
        return 0;
    }

    let mut reports = 0;

    if count_color_runs(lamps) <= 4 {
        let mut start = 0;
        while start < lamp_count {
            let color = lamps[start];
            let mut end = start + 1;
            while end < lamp_count && lamps[end] == color {
                end += 1;
            }

            let mut buffer = [0u8; LAMP_ARRAY_FEATURE_LEN];
            buffer[0] = LAMP_ARRAY_RANGE_UPDATE_REPORT;
            buffer[1] = if end >= lamp_count { 0x01 } else { 0x00 };
            buffer[2] = start as u8;
            buffer[3] = (start >> 8) as u8;
            buffer[4] = (end - 1) as u8;
            buffer[5] = ((end - 1) >> 8) as u8;
            buffer[6] = color[0];
            buffer[7] = color[1];
            buffer[8] = color[2];
            buffer[9] = intensity;

            let _ = device.send_feature_report(&buffer);
            reports += 1;
            start = end;
        }
    } else {
        let mut lamp = 0;
        while lamp < lamp_count {
            let chunk = (lamp_count - lamp).min(8);
            let complete = lamp + chunk >= lamp_count;

            let mut buffer = [0u8; LAMP_ARRAY_FEATURE_LEN];
            buffer[0] = LAMP_ARRAY_MULTI_UPDATE_REPORT;
            buffer[1] = chunk as u8;
            buffer[2] = if complete { 0x01 } else { 0x00 };

            for i in 0..chunk {
                let id = (lamp + i) as u16;
                buffer[3 + i * 2] = id as u8;
                buffer[4 + i * 2] = (id >> 8) as u8;

                let color = lamps[lamp + i];
                let offset = 19 + i * 4;
                buffer[offset] = color[0];
                buffer[offset + 1] = color[1];
                buffer[offset + 2] = color[2];
                buffer[offset + 3] = intensity;
            }

            let _ = device.send_feature_report(&buffer);
            reports += 1;
            lamp += chunk;
        }
    }

    reports
}

fn frame_delay_ms(reports: usize) -> u64 {
    match reports {
        0 | 1 => 16,
        2 => 22,
        _ => 33,
    }
}

fn spawn_lamp_array_worker(
    device: HidDevice,
    lamp_count: usize,
    state: Arc<Mutex<LightingState>>,
    stop: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let started = Instant::now();
        let mut last_reclaim = started;

        loop {
            let stopping = stop.load(Ordering::SeqCst);

            let current_state = match state.lock() {
                Ok(state) => *state,
                Err(_) => break,
            };

            // Windows can take the LampArray back after a resume or a lighting
            // service restart. Re-claim host control before painting.
            if started.elapsed() < Duration::from_secs(20) || last_reclaim.elapsed() >= Duration::from_secs(1) {
                send_lamp_array_control(&device, 0x00);
                last_reclaim = Instant::now();
            }

            let elapsed = started.elapsed().as_secs_f64();
            let lamps = compute_lamps(&current_state, lamp_count, elapsed);
            let intensity = if current_state.brightness <= 1 { 128 } else { 255 };
            let reports = send_lamps(&device, &lamps, intensity);

            if stopping {
                // Keep the last frame on the keyboard. Closing the HID handle
                // without releasing control makes the device retain it, which
                // is what lets CLI `set` and a closed GUI keep their colors.
                break;
            }

            thread::sleep(Duration::from_millis(frame_delay_ms(reports)));
        }
    })
}

fn open_lamp_array_keyboard(
    api: &HidApi,
    info: &hidapi::DeviceInfo,
    stop_signal: Arc<AtomicBool>,
    lighting_restore: &mut Option<LightingRestore>,
) -> Result<Keyboard> {
    #[cfg(not(target_os = "windows"))]
    let _ = lighting_restore;

    let device = info.open_device(api)?;

    let mut attributes = [0u8; LAMP_ARRAY_FEATURE_LEN];
    attributes[0] = LAMP_ARRAY_ATTRIBUTES_REPORT;
    let lamp_count = match device.get_feature_report(&mut attributes) {
        Ok(length) if length >= 3 => {
            let count = usize::from(u16::from_le_bytes([attributes[1], attributes[2]]));
            if count > 0 {
                count
            } else {
                LOQ_15IRX10_LAMP_COUNT
            }
        }
        _ => LOQ_15IRX10_LAMP_COUNT,
    };

    // Take the lamps away from the device's own effect engine.
    send_lamp_array_control(&device, 0x00);

    let current_state = LightingState {
        effect_type: BaseEffects::Static,
        speed: 1,
        brightness: 1,
        rgb_values: [0; 12],
    };
    let shared_state = Arc::new(Mutex::new(current_state));
    let lamp_stop = Arc::new(AtomicBool::new(false));
    let worker = spawn_lamp_array_worker(device, lamp_count, shared_state.clone(), lamp_stop.clone());

    Ok(Keyboard {
        keyboard_hid: None,
        current_state,
        stop_signal,
        protocol: Protocol::LampArray,
        lamp: Some(LampArrayHandle {
            state: shared_state,
            stop: lamp_stop,
            thread: Some(worker),
        }),
        #[cfg(target_os = "windows")]
        lighting_restore: lighting_restore.take(),
    })
}

#[cfg(target_os = "windows")]
mod windows_lighting {
    use std::{
        process::Output,
        thread,
        time::Duration,
    };

    const LIGHTING_KEY: &str = r"HKCU\Software\Microsoft\Lighting";
    const DEVICES_KEY: &str = r"HKCU\Software\Microsoft\Lighting\Devices";
    const PROVIDERS_KEY: &str = r"HKCU\Software\Microsoft\Lighting\Providers";

    #[derive(Clone)]
    struct SavedDword {
        key: String,
        name: String,
        value: Option<u32>,
    }

    /// The values changed while this app owns the keyboard. Restoring them on
    /// drop hands the keyboard back to Windows Dynamic Lighting.
    pub struct LightingRestore {
        values: Vec<SavedDword>,
    }

    impl LightingRestore {
        fn capture() -> Self {
            let mut values = Vec::new();
            let mut keys = vec![LIGHTING_KEY.to_string()];
            keys.extend(reg_subkeys(DEVICES_KEY));

            for key in keys {
                for name in ["AmbientLightingEnabled", "ControlledByForegroundApp", "UseSystemAccentColor"] {
                    values.push(SavedDword {
                        key: key.clone(),
                        name: name.to_string(),
                        value: reg_query_dword(&key, name),
                    });
                }
            }

            Self { values }
        }
    }

    impl Drop for LightingRestore {
        fn drop(&mut self) {
            for saved in &self.values {
                match saved.value {
                    Some(value) => {
                        reg_set_dword(&saved.key, &saved.name, value);
                    }
                    None => {
                        reg_delete_value(&saved.key, &saved.name);
                    }
                }
            }
        }
    }

    fn run_hidden(program: &str, args: &[&str]) -> Option<Output> {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        std::process::Command::new(program)
            .args(args)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()
    }

    fn reg_subkeys(key: &str) -> Vec<String> {
        let Some(output) = run_hidden("reg", &["query", key]) else {
            return Vec::new();
        };

        let expanded = key.replace("HKCU", "HKEY_CURRENT_USER").replace("HKLM", "HKEY_LOCAL_MACHINE");
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("HKEY_") && !line.eq_ignore_ascii_case(&expanded))
            .map(str::to_string)
            .collect()
    }

    fn reg_query_dword(key: &str, name: &str) -> Option<u32> {
        let output = run_hidden("reg", &["query", key, "/v", name])?;
        if !output.status.success() {
            return None;
        }

        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[0].eq_ignore_ascii_case(name) && parts[1].eq_ignore_ascii_case("REG_DWORD") {
                let hex = parts[2].trim_start_matches("0x").trim_start_matches("0X");
                return u32::from_str_radix(hex, 16).ok();
            }
        }

        None
    }

    fn reg_set_dword(key: &str, name: &str, value: u32) -> bool {
        run_hidden("reg", &["add", key, "/v", name, "/t", "REG_DWORD", "/d", &value.to_string(), "/f"])
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn reg_delete_value(key: &str, name: &str) -> bool {
        run_hidden("reg", &["delete", key, "/v", name, "/f"])
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn set_ambient_lighting(value: u32) {
        reg_set_dword(LIGHTING_KEY, "AmbientLightingEnabled", value);
        for device_key in reg_subkeys(DEVICES_KEY) {
            reg_set_dword(&device_key, "AmbientLightingEnabled", value);
        }
    }

    fn remove_legion_providers(key: &str) {
        let Some(output) = run_hidden("reg", &["query", key]) else {
            return;
        };

        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if !line.to_ascii_lowercase().contains("legionlighting") {
                continue;
            }
            if let Some(name) = line.split_whitespace().next() {
                reg_delete_value(key, name);
            }
        }
    }

    fn stop_legion_lighting_helpers() {
        for image in ["LegionLightingController.exe", "LenovoLighting.exe", "LenovoLightingService.exe"] {
            let _ = run_hidden("taskkill", &["/IM", image, "/F"]);
        }
    }

    /// Make Windows Dynamic Lighting let go of the keyboard. Windows only
    /// releases the LampArray when `AmbientLightingEnabled` changes, so toggle
    /// it on and back off instead of writing 0 again.
    pub fn prepare() -> LightingRestore {
        let restore = LightingRestore::capture();

        stop_legion_lighting_helpers();
        remove_legion_providers(PROVIDERS_KEY);
        for device_key in reg_subkeys(DEVICES_KEY) {
            remove_legion_providers(&format!(r"{}\Providers", device_key));
        }

        reg_set_dword(LIGHTING_KEY, "ControlledByForegroundApp", 0);
        reg_set_dword(LIGHTING_KEY, "UseSystemAccentColor", 0);
        for device_key in reg_subkeys(DEVICES_KEY) {
            reg_set_dword(&device_key, "ControlledByForegroundApp", 0);
            reg_set_dword(&device_key, "UseSystemAccentColor", 0);
        }

        set_ambient_lighting(1);
        thread::sleep(Duration::from_millis(700));
        set_ambient_lighting(0);
        thread::sleep(Duration::from_millis(1000));

        restore
    }
}

#[cfg(target_os = "windows")]
use windows_lighting::LightingRestore;

#[cfg(not(target_os = "windows"))]
struct LightingRestore;

pub fn get_keyboard(stop_signal: Arc<AtomicBool>) -> Result<Keyboard> {
    let mut api: HidApi = HidApi::new()?;

    if api.device_list().any(is_loq_15irx10) {
        #[cfg(target_os = "windows")]
        let mut lighting_restore = Some(windows_lighting::prepare());
        #[cfg(not(target_os = "windows"))]
        let mut lighting_restore: Option<LightingRestore> = None;

        // The lighting service can re-enumerate the device while it releases
        // it, so retry for a few seconds.
        for attempt in 0..16 {
            if attempt > 0 {
                thread::sleep(Duration::from_millis(500));
                let _ = api.refresh_devices();
            }

            let Some(info) = api.device_list().find(|info| is_loq_15irx10(info)) else {
                continue;
            };

            let result = open_lamp_array_keyboard(
                &api,
                info,
                stop_signal.clone(),
                &mut lighting_restore,
            );

            if let Ok(keyboard) = result {
                return Ok(keyboard);
            }
        }

        // Dropping the restore object puts the Windows lighting settings back.
        // If LampArray is unavailable, fall through and try the legacy vendor
        // collection instead of failing outright.
        drop(lighting_restore);
    }

    let info = api
        .device_list()
        .find(|d| {
            #[cfg(target_os = "windows")]
            {
                let info_tuple = (d.vendor_id(), d.product_id(), d.usage_page(), d.usage());

                KNOWN_DEVICE_INFOS.iter().any(|known| known == &info_tuple)
            }

            #[cfg(target_os = "linux")]
            {
                let info_tuple = (d.vendor_id(), d.product_id());

                KNOWN_DEVICE_INFOS.iter().any(|known| (known.0, known.1) == info_tuple)
            }
        })
        .ok_or(error::Error::DeviceNotFound)?;

    let keyboard_hid: HidDevice = info.open_device(&api)?;
    let current_state: LightingState = LightingState {
        effect_type: BaseEffects::Static,
        speed: 1,
        brightness: 1,
        rgb_values: [0; 12],
    };

    let mut keyboard = Keyboard {
        keyboard_hid: Some(keyboard_hid),
        current_state,
        stop_signal,
        protocol: Protocol::Legacy,
        lamp: None,
        #[cfg(target_os = "windows")]
        lighting_restore: None,
    };

    keyboard.refresh()?;
    Ok(keyboard)
}

pub fn find_possible_keyboards() -> Result<Vec<String>> {
    let api: HidApi = HidApi::new()?;

    let mut list = api
        .device_list()
        .filter(|d| d.vendor_id() == 0x048d)
        .map(|d| format!("{:#06x}:{:#06x}", d.vendor_id(), d.product_id()))
        .collect::<Vec<String>>();

    list.sort();
    list.dedup();
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZONES: [u8; 12] = [
        255, 0, 0, // red
        0, 255, 0, // green
        0, 0, 255, // blue
        255, 255, 255, // white
    ];

    #[test]
    fn four_zones_map_to_six_lamps_each() {
        let lamps = expand_zones_to_lamps(&ZONES, 24);
        assert_eq!(lamps.len(), 24);
        assert!(lamps[0..6].iter().all(|lamp| *lamp == [255, 0, 0]));
        assert!(lamps[6..12].iter().all(|lamp| *lamp == [0, 255, 0]));
        assert!(lamps[12..18].iter().all(|lamp| *lamp == [0, 0, 255]));
        assert!(lamps[18..24].iter().all(|lamp| *lamp == [255, 255, 255]));
    }

    #[test]
    fn single_lamp_uses_first_zone() {
        let lamps = expand_zones_to_lamps(&ZONES, 1);
        assert_eq!(lamps, vec![[255, 0, 0]]);
    }

    #[test]
    fn four_zones_need_four_range_updates() {
        let lamps = expand_zones_to_lamps(&ZONES, 24);
        assert_eq!(count_color_runs(&lamps), 4);
    }

    #[test]
    fn gradient_needs_multi_updates() {
        let lamps = compute_lamps(
            &LightingState {
                effect_type: BaseEffects::Smooth,
                speed: 1,
                brightness: 2,
                rgb_values: ZONES,
            },
            24,
            1.0,
        );
        assert!(count_color_runs(&lamps) > 4);
    }

    #[test]
    fn static_effect_keeps_zone_colors() {
        let lamps = compute_lamps(
            &LightingState {
                effect_type: BaseEffects::Static,
                speed: 1,
                brightness: 1,
                rgb_values: ZONES,
            },
            24,
            0.0,
        );
        assert_eq!(lamps[0], [255, 0, 0]);
        assert_eq!(lamps[23], [255, 255, 255]);
    }

    #[test]
    fn brightness_scales_all_channels() {
        assert_eq!(scale_rgb([200, 100, 50], 0.5), [100, 50, 25]);
        assert_eq!(scale_rgb([200, 100, 50], 1.0), [200, 100, 50]);
    }
}
