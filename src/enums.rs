use crate::{effects::custom_effect::CustomEffect, profile::Profile};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Clone, Copy, EnumString, Serialize, Deserialize, Display, EnumIter, PartialEq, Eq, Debug)]
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

#[allow(dead_code)]
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
}

#[derive(Clone, Copy, EnumString, Serialize, Deserialize, Debug)]
pub enum Direction {
	Left,
	Right,
}

impl Default for Direction {
	fn default() -> Self {
		Self::Left
	}
}

#[derive(Debug)]
pub enum Message {
	CustomEffect { effect: CustomEffect },
	Refresh,
	Profile { profile: Profile },
	Exit,
}
