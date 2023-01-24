use crate::{effects::custom_effect::CustomEffect, profile::Profile};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

#[derive(Clone, Copy, EnumString, Serialize, Deserialize, Display, EnumIter, Eq, Debug, IntoStaticStr, Default)]
pub enum Effects {
    #[default]
    Static,
    Breath,
    Smooth,
    Wave,
    Lightning,
    AmbientLight {
        fps: u8,
    },
    SmoothWave,
    Swipe,
    Disco,
    Christmas,
    Fade,
    Temperature,
    Ripple,
}

impl PartialEq for Effects {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

#[allow(dead_code)]
impl Effects {
    pub fn takes_color_array(self) -> bool {
        matches!(self, Self::Static | Self::Breath | Self::Swipe { .. } | Self::Fade | Self::Ripple)
    }

    pub fn takes_direction(self) -> bool {
        matches!(self, Self::Wave | Self::SmoothWave | Self::Swipe { .. })
    }

    pub fn takes_speed(self) -> bool {
        matches!(
            self,
            Self::Breath | Self::Smooth | Self::Wave | Self::Lightning | Self::SmoothWave | Self::Swipe | Self::Disco | Self::Fade | Self::Ripple
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

#[derive(Debug)]
pub enum Message {
    CustomEffect { effect: CustomEffect },
    Profile { profile: Profile },
    Exit,
}
