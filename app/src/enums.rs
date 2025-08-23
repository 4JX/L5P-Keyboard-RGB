use crate::manager::{custom_effect::CustomEffect, profile::Profile};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};
use xcap::Monitor;

#[derive(Clone, Copy, EnumString, Serialize, Deserialize, Display, EnumIter, Debug, IntoStaticStr, Default)]
pub enum Effects {
    #[default]
    Static,
    Breath,
    Smooth,
    Wave,
    Lightning,
    AmbientLight {
        monitor_id: MonitorId,
        fps: u8,
        saturation_boost: f32,
    },
    SmoothWave {
        mode: SwipeMode,
        clean_with_black: bool,
    },
    Swipe {
        mode: SwipeMode,
        clean_with_black: bool,
    },
    Disco,
    Christmas,
    Fade,
    Temperature,
    Ripple,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MonitorId(pub u32);

impl Default for MonitorId {
    fn default() -> Self {
        if let Ok(monitors) = Monitor::all() {
            if let Some(primary) = monitors.iter().find(|m| m.is_primary().unwrap_or(false)) {
                MonitorId(primary.id().unwrap())
            } else {
                MonitorId(0)
            }
        } else {
            MonitorId(0)
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, EnumIter, EnumString, PartialEq)]
pub enum SwipeMode {
    #[default]
    Change,
    Fill,
}

impl PartialEq for Effects {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

#[allow(dead_code)]
impl Effects {
    pub fn takes_color_array(self) -> bool {
        matches!(self, Self::Static | Self::Breath | Self::Lightning | Self::Swipe { .. } | Self::Fade | Self::Ripple)
    }

    pub fn takes_direction(self) -> bool {
        matches!(self, Self::Wave | Self::SmoothWave { .. } | Self::Swipe { .. })
    }

    pub fn takes_speed(self) -> bool {
        matches!(
            self,
            Self::Breath | Self::Smooth | Self::Wave | Self::Lightning | Self::SmoothWave { .. } | Self::Swipe { .. } | Self::Disco | Self::Fade | Self::Ripple
        )
    }

    pub fn is_built_in(self) -> bool {
        matches!(self, Self::Static | Self::Breath | Self::Smooth | Self::Wave)
    }
}

#[derive(Clone, Copy, EnumString, Serialize, Deserialize, Debug, EnumIter, IntoStaticStr, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    Left,
    Right,
}

#[derive(PartialEq, Eq, EnumIter, IntoStaticStr, Clone, Copy, Default, Serialize, Deserialize, Debug, Display, EnumString)]
pub enum Brightness {
    #[default]
    Low,
    High,
}

#[derive(Debug)]
pub enum Message {
    CustomEffect { effect: CustomEffect },
    Profile { profile: Profile },
    Exit,
}
