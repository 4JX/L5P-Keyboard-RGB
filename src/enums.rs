use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Clone, Copy, EnumString, Serialize, Deserialize, Display)]
pub enum Effects {
	Static,
	Breath,
	Smooth,
	LeftWave,
	RightWave,
	Lightning,
	AmbientLight,
	SmoothLeftWave,
	SmoothRightWave,
	LeftSwipe,
	RightSwipe,
	Disco,
	Christmas,
	Fade,
	Temperature,
}

#[allow(dead_code)]
pub enum Message {
	UpdateAllValues { value: [u8; 12] },
	UpdateEffect { effect: Effects },
	UpdateBrightness { brightness: u8 },
	UpdateSpeed { speed: u8 },
	Refresh,
}
