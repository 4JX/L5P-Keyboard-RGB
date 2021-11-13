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

pub enum Message {
	UpdateAllValues { value: [f32; 12] },
	UpdateRGB { index: u8, value: f32 },
	UpdateZone { zone_index: u8, value: [f32; 3] },
	UpdateEffect { effect: Effects },
	UpdateValue { index: u8, value: f32 },
	UpdateBrightness { brightness: u8 },
	UpdateSpeed { speed: u8 },
	Restart,
}
