use strum_macros::EnumString;

#[derive(Clone, Copy, EnumString)]
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
}

#[allow(dead_code)]
pub enum Message {
	UpdateAllValues { value: [u8; 12] },
	UpdateRGB { index: u8, value: u8 },
	UpdateZone { zone_index: u8, value: [u8; 3] },
	UpdateEffect { effect: Effects },
	UpdateValue { index: u8, value: u8 },
	UpdateBrightness { brightness: u8 },
	UpdateSpeed { speed: u8 },
	Refresh,
}
