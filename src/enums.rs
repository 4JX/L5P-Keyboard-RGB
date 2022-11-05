use crate::custom_effect::CustomEffect;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Clone, Copy, EnumString, Serialize, Deserialize, Display, EnumIter, PartialEq, Eq)]
pub enum Effects {
	Static,
	Breath,
	Smooth,
	Wave,
	Lightning,
	AmbientLight { fps: u8 },
	SmoothWave,
	Swipe,
	Disco,
	Christmas,
	Fade,
	Temperature,
	Ripple,
}

impl Default for Effects {
	fn default() -> Self {
		Self::Static
	}
}

impl Effects {
	pub fn takes_color_array(&self) -> bool {
		matches!(self, Effects::Static | Effects::Breath | Effects::Swipe { .. } | Effects::Fade | Effects::Ripple)
	}

	pub fn takes_direction(&self) -> bool {
		matches!(self, Effects::Wave | Effects::SmoothWave | Effects::Swipe { .. })
	}

	pub fn takes_speed(&self) -> bool {
		matches!(
			self,
			Effects::Breath | Effects::Smooth | Effects::Wave | Effects::Lightning | Effects::SmoothWave | Effects::Swipe | Effects::Disco | Effects::Fade | Effects::Ripple
		)
	}

	pub fn as_u8(&self) -> u8 {
		match self {
			Effects::Static => 0,
			Effects::Breath => 1,
			Effects::Smooth => 2,
			Effects::Wave => 3,
			Effects::Lightning => 4,
			Effects::AmbientLight { .. } => 5,
			Effects::SmoothWave => 6,
			Effects::Swipe => 7,
			Effects::Disco => 8,
			Effects::Christmas => 9,
			Effects::Fade => 10,
			Effects::Temperature => 11,
			Effects::Ripple => 12,
		}
	}
}

#[derive(Clone, Copy, EnumString, Serialize, Deserialize)]
pub enum Direction {
	Left,
	Right,
}

impl Default for Direction {
	fn default() -> Self {
		Self::Left
	}
}

pub enum Message {
	CustomEffect { effect: CustomEffect },
	Refresh,
}
