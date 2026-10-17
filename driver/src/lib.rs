use error::{RangeError, RangeErrorKind, Result};
use hidapi::{HidApi, HidDevice};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

pub mod error;

/// Keyboards using the original 4 zone protocol (33 byte feature report, report ID 0xcc).
const LEGACY_DEVICE_INFOS: [(u16, u16, u16, u16); 11] = [
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
];

/// Keyboards using the "Spectrum" protocol (960 byte feature report, report ID 0x07), as found on the
/// 2025 lineup. The LED layout is read from the device and grouped into 4 zones by column.
const SPECTRUM_DEVICE_INFOS: [(u16, u16, u16, u16); 1] = [
    (0x048d, 0xc195, 0xff89, 0x0007), // 2025
];

pub const SPEED_RANGE: std::ops::RangeInclusive<u8> = 1..=4;
pub const BRIGHTNESS_RANGE: std::ops::RangeInclusive<u8> = 1..=2;
pub const ZONE_RANGE: std::ops::RangeInclusive<u8> = 0..=3;

const SPECTRUM_REPORT_ID: u8 = 0x07;
const SPECTRUM_REPORT_SIZE: usize = 960;

mod spectrum_op {
    pub const KEY_COUNT: u8 = 0xc4;
    pub const KEY_PAGE: u8 = 0xc5;
    pub const PROFILE: u8 = 0xca;
    pub const EFFECT_CHANGE: u8 = 0xcb;
    pub const BRIGHTNESS: u8 = 0xce;
}

mod spectrum_effect {
    pub const SCREW_RAINBOW: u8 = 1;
    pub const RAINBOW_WAVE: u8 = 2;
    pub const COLOR_CHANGE: u8 = 3;
    pub const COLOR_PULSE: u8 = 4;
    pub const COLOR_WAVE: u8 = 5;
    pub const SMOOTH: u8 = 6;
    pub const RAIN: u8 = 7;
    pub const ALWAYS: u8 = 11;

    pub const COLOR_MODE_NONE: u8 = 0;
    pub const COLOR_MODE_RANDOM: u8 = 1;
    pub const COLOR_MODE_LIST: u8 = 2;

    pub const DIRECTION_TOP_TO_BOTTOM: u8 = 2;
    pub const DIRECTION_RIGHT_TO_LEFT: u8 = 3;
    pub const DIRECTION_LEFT_TO_RIGHT: u8 = 4;

    pub const CLOCKWISE: u8 = 1;
    pub const COUNTER_CLOCKWISE: u8 = 2;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BaseEffects {
    Static,
    Breath,
    Smooth,
    LeftWave,
    RightWave,
    // The effects below only exist on "Spectrum" keyboards (2025+). On older keyboards they fall back to Static.
    /// Rainbow spiral, rotating clockwise or counter clockwise
    RainbowScrewClockwise,
    RainbowScrewCounterClockwise,
    /// Cycles through the zone colors (or random colors when none are set)
    ColorChange,
    /// A wave of the zone colors (or random colors when none are set) moving left or right
    ColorWaveLeft,
    ColorWaveRight,
    /// Raindrops in the zone colors (or random colors when none are set)
    Rain,
}

impl BaseEffects {
    /// Whether this effect needs a "Spectrum" keyboard
    pub fn is_spectrum_only(self) -> bool {
        !matches!(self, Self::Static | Self::Breath | Self::Smooth | Self::LeftWave | Self::RightWave)
    }

    /// Whether this effect uses the zone colors
    pub fn takes_colors(self) -> bool {
        matches!(self, Self::Static | Self::Breath | Self::ColorChange | Self::ColorWaveLeft | Self::ColorWaveRight | Self::Rain)
    }
}

pub struct LightingState {
    effect_type: BaseEffects,
    speed: u8,
    brightness: u8,
    rgb_values: [u8; 12],
}

enum Backend {
    Legacy,
    Spectrum {
        profile: u8,
        /// Key codes of the LEDs belonging to each of the 4 zones, left to right.
        zone_keys: [Vec<u16>; 4],
    },
}

pub struct Keyboard {
    keyboard_hid: HidDevice,
    current_state: LightingState,
    stop_signal: Arc<AtomicBool>,
    backend: Backend,
}

#[allow(dead_code)]
impl Keyboard {
    fn validate_state(&self) -> Result<()> {
        let keyboard_state = &self.current_state;

        if !SPEED_RANGE.contains(&keyboard_state.speed) {
            return Err(RangeError { kind: RangeErrorKind::Speed }.into());
        }
        if !BRIGHTNESS_RANGE.contains(&keyboard_state.brightness) {
            return Err(RangeError { kind: RangeErrorKind::Brightness }.into());
        }

        Ok(())
    }

    fn build_payload(&self) -> Result<[u8; 33]> {
        self.validate_state()?;
        let keyboard_state = &self.current_state;

        let mut payload: [u8; 33] = [0; 33];
        payload[0] = 0xcc;
        payload[1] = 0x16;
        payload[2] = match keyboard_state.effect_type {
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
            // Static, and the Spectrum-only effects which fall back to it
            _ => 0x01,
        };

        payload[3] = keyboard_state.speed;
        payload[4] = keyboard_state.brightness;

        if !matches!(keyboard_state.effect_type, BaseEffects::Smooth | BaseEffects::LeftWave | BaseEffects::RightWave) {
            payload[5..(12 + 5)].copy_from_slice(&keyboard_state.rgb_values[..12]);
        };

        Ok(payload)
    }

    fn build_spectrum_payload(&self, profile: u8, zone_keys: &[Vec<u16>; 4]) -> Result<Vec<u8>> {
        self.validate_state()?;
        let keyboard_state = &self.current_state;

        // The Spectrum protocol only knows 3 speeds
        let speed = keyboard_state.speed.min(3);

        let mut payload = vec![profile, 0x01, 0x01];

        let mut push_effect = |effect_no: u8, effect_type: u8, direction: u8, clockwise: u8, color_mode: u8, colors: &[[u8; 3]], keys: &[u16]| {
            payload.extend_from_slice(&[effect_no, 0x06, 0x01, effect_type, 0x02, speed, 0x03, clockwise, 0x04, direction, 0x05, color_mode, 0x06, 0x00]);
            payload.push(colors.len() as u8);
            for color in colors {
                payload.extend_from_slice(color);
            }
            payload.push(keys.len() as u8);
            for key in keys {
                payload.extend_from_slice(&key.to_le_bytes());
            }
        };

        let all_keys: Vec<u16> = zone_keys.iter().flatten().copied().collect();

        // The distinct, non-black zone colors, used as the color list for the effects that take one.
        // Without any colors the keyboard picks random ones instead.
        let mut color_list: Vec<[u8; 3]> = Vec::with_capacity(4);
        for zone_index in 0..4 {
            let rgb = &keyboard_state.rgb_values;
            let color = [rgb[zone_index * 3], rgb[zone_index * 3 + 1], rgb[zone_index * 3 + 2]];
            if color != [0, 0, 0] && !color_list.contains(&color) {
                color_list.push(color);
            }
        }
        let color_mode = if color_list.is_empty() {
            spectrum_effect::COLOR_MODE_RANDOM
        } else {
            spectrum_effect::COLOR_MODE_LIST
        };

        use spectrum_effect::*;
        match keyboard_state.effect_type {
            BaseEffects::Static | BaseEffects::Breath => {
                let effect_type = if let BaseEffects::Static = keyboard_state.effect_type { ALWAYS } else { COLOR_PULSE };

                for (zone_index, keys) in zone_keys.iter().enumerate() {
                    let rgb = &keyboard_state.rgb_values;
                    let color = [rgb[zone_index * 3], rgb[zone_index * 3 + 1], rgb[zone_index * 3 + 2]];
                    push_effect(zone_index as u8 + 1, effect_type, 0, 0, COLOR_MODE_LIST, &[color], keys);
                }
            }
            BaseEffects::Smooth => push_effect(1, SMOOTH, 0, 0, COLOR_MODE_RANDOM, &[], &all_keys),
            BaseEffects::LeftWave => push_effect(1, RAINBOW_WAVE, DIRECTION_RIGHT_TO_LEFT, 0, COLOR_MODE_NONE, &[], &all_keys),
            BaseEffects::RightWave => push_effect(1, RAINBOW_WAVE, DIRECTION_LEFT_TO_RIGHT, 0, COLOR_MODE_NONE, &[], &all_keys),
            BaseEffects::RainbowScrewClockwise => push_effect(1, SCREW_RAINBOW, 0, CLOCKWISE, COLOR_MODE_NONE, &[], &all_keys),
            BaseEffects::RainbowScrewCounterClockwise => push_effect(1, SCREW_RAINBOW, 0, COUNTER_CLOCKWISE, COLOR_MODE_NONE, &[], &all_keys),
            BaseEffects::ColorChange => push_effect(1, COLOR_CHANGE, 0, 0, color_mode, &color_list, &all_keys),
            BaseEffects::ColorWaveLeft => push_effect(1, COLOR_WAVE, DIRECTION_RIGHT_TO_LEFT, 0, color_mode, &color_list, &all_keys),
            BaseEffects::ColorWaveRight => push_effect(1, COLOR_WAVE, DIRECTION_LEFT_TO_RIGHT, 0, color_mode, &color_list, &all_keys),
            BaseEffects::Rain => push_effect(1, RAIN, DIRECTION_TOP_TO_BOTTOM, 0, color_mode, &color_list, &all_keys),
        }

        Ok(spectrum_request(spectrum_op::EFFECT_CHANGE, &payload))
    }

    fn spectrum_send_brightness(&self) -> Result<()> {
        // Map Low/High onto the Spectrum 0..=9 scale
        let brightness = match self.current_state.brightness {
            1 => 5,
            _ => 9,
        };
        self.keyboard_hid.send_feature_report(&spectrum_request(spectrum_op::BRIGHTNESS, &[brightness]))?;
        Ok(())
    }

    /// Whether the keyboard supports the "Spectrum" only effects (see [`BaseEffects::is_spectrum_only`])
    pub fn supports_spectrum_effects(&self) -> bool {
        matches!(self.backend, Backend::Spectrum { .. })
    }

    pub fn refresh(&mut self) -> Result<()> {
        match &self.backend {
            Backend::Legacy => {
                let payload = self.build_payload()?;
                self.keyboard_hid.send_feature_report(&payload)?;
            }
            Backend::Spectrum { profile, zone_keys } => {
                let payload = self.build_spectrum_payload(*profile, zone_keys)?;
                self.keyboard_hid.send_feature_report(&payload)?;
            }
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

        if let Backend::Spectrum { .. } = self.backend {
            self.spectrum_send_brightness()?;
        }
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
        if self.current_state.effect_type.takes_colors() {
            for (i, _) in new_values.iter().enumerate() {
                self.current_state.rgb_values[i] = new_values[i];
            }
            self.refresh()?;
        }

        Ok(())
    }

    pub fn solid_set_colors_to(&mut self, new_values: [u8; 3]) -> Result<()> {
        if self.current_state.effect_type.takes_colors() {
            for i in (0..12).step_by(3) {
                self.current_state.rgb_values[i] = new_values[0];
                self.current_state.rgb_values[i + 1] = new_values[1];
                self.current_state.rgb_values[i + 2] = new_values[2];
            }
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

/// Builds a full sized Spectrum feature report: `[0x07, operation, size, 0x03]` followed by the payload.
fn spectrum_request(operation: u8, payload: &[u8]) -> Vec<u8> {
    let mut request = vec![SPECTRUM_REPORT_ID, operation, 0xc0, 0x03];
    request.extend_from_slice(payload);
    request.resize(SPECTRUM_REPORT_SIZE, 0);
    request
}

/// Sends a Spectrum request and reads back the device's response to it.
fn spectrum_query(device: &HidDevice, operation: u8, payload: &[u8]) -> Result<Vec<u8>> {
    device.send_feature_report(&spectrum_request(operation, payload))?;
    let mut response = vec![0; SPECTRUM_REPORT_SIZE];
    response[0] = SPECTRUM_REPORT_ID;
    device.get_feature_report(&mut response)?;
    Ok(response)
}

/// Reads the LED layout from a Spectrum keyboard and splits it into 4 zones by column.
///
/// The device reports its LEDs as a matrix of `rows` pages with `cols` entries each. Every entry is
/// `[index, key code (u16 LE)]`; a key code of 0 means there is no LED at that position. Key codes below
/// 0x0100 are the keyboard itself, higher ones are accent lighting (chassis, logo) and are left alone.
fn spectrum_read_zones(device: &HidDevice) -> Result<[Vec<u16>; 4]> {
    let key_count = spectrum_query(device, spectrum_op::KEY_COUNT, &[0x07])?;
    let (rows, cols) = (key_count[5] as usize, key_count[6] as usize);
    if rows == 0 || cols == 0 {
        return Err(error::Error::KeyMapUnavailable);
    }

    let mut zone_keys: [Vec<u16>; 4] = Default::default();
    for row in 0..rows {
        let page = spectrum_query(device, spectrum_op::KEY_PAGE, &[0x07, row as u8])?;
        for col in 0..cols {
            let offset = 6 + col * 3;
            let key_code = u16::from_le_bytes([page[offset + 1], page[offset + 2]]);
            if key_code == 0 || key_code >= 0x0100 {
                continue;
            }
            let zone = (col * ZONE_RANGE.count()) / cols;
            zone_keys[zone].push(key_code);
        }
    }

    if zone_keys.iter().any(Vec::is_empty) {
        return Err(error::Error::KeyMapUnavailable);
    }

    Ok(zone_keys)
}

fn matches_known(d: &hidapi::DeviceInfo, known_infos: &[(u16, u16, u16, u16)]) -> bool {
    #[cfg(target_os = "windows")]
    {
        let info_tuple = (d.vendor_id(), d.product_id(), d.usage_page(), d.usage());

        known_infos.iter().any(|known| known == &info_tuple)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let info_tuple = (d.vendor_id(), d.product_id());

        known_infos.iter().any(|known| (known.0, known.1) == info_tuple)
    }
}

pub fn get_keyboard(stop_signal: Arc<AtomicBool>) -> Result<Keyboard> {
    let api: HidApi = HidApi::new()?;

    let (info, is_spectrum) = api
        .device_list()
        .find_map(|d| {
            if matches_known(d, &LEGACY_DEVICE_INFOS) {
                Some((d, false))
            } else if matches_known(d, &SPECTRUM_DEVICE_INFOS) && (cfg!(target_os = "windows") || d.interface_number() == 0) {
                // The Spectrum report lives on the first HID interface, the second one is Dynamic Lighting
                Some((d, true))
            } else {
                None
            }
        })
        .ok_or(error::Error::DeviceNotFound)?;

    let keyboard_hid: HidDevice = info.open_device(&api)?;

    let backend = if is_spectrum {
        let profile = spectrum_query(&keyboard_hid, spectrum_op::PROFILE, &[])?[4];
        let zone_keys = spectrum_read_zones(&keyboard_hid)?;
        Backend::Spectrum { profile, zone_keys }
    } else {
        Backend::Legacy
    };

    let current_state: LightingState = LightingState {
        effect_type: BaseEffects::Static,
        speed: 1,
        brightness: 1,
        rgb_values: [0; 12],
    };

    let mut keyboard = Keyboard {
        keyboard_hid,
        current_state,
        stop_signal,
        backend,
    };

    if let Backend::Spectrum { .. } = keyboard.backend {
        keyboard.spectrum_send_brightness()?;
    }
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

    list.dedup();
    Ok(list)
}
