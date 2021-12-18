use crate::custom_effect::CustomEffect;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Clone, Copy, EnumString, Serialize, Deserialize, Display, EnumIter, PartialEq)]
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
}

#[derive(Clone, Copy, EnumString, Serialize, Deserialize)]
pub enum Direction {
	Left,
	Right,
}

pub enum Message {
	UpdateEffect { effect: Effects },
	CustomEffect { effect: CustomEffect },
	Refresh,
}
