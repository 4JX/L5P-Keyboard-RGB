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
	AmbientLight,
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
