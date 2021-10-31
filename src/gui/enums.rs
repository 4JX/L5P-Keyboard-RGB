#[derive(Clone, Copy)]
pub enum BaseColor {
	Red,
	Green,
	Blue,
}

#[derive(Clone, Copy)]
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
}

pub enum Colors {
	Red = 0xff0000,
	Green = 0x00ff00,
	Blue = 0x0000ff,
	DarkerGray = 0x222222,
	DarkGray = 0x333333,
	Gray = 0x444444,
	LightGray = 0x777777,
	LighterGray = 0xcccccc,
	White = 0xffffff,
}

pub enum Message {
	UpdateAllValues { value: [f32; 12] },
	UpdateRGB { index: u8, value: f32 },
	UpdateZone { zone_index: u8, value: [f32; 3] },
	UpdateEffect { effect: Effects },
	UpdateValue { index: u8, value: f32 },
	UpdateBrightness { brightness: u8 },
	UpdateSpeed { speed: u8 },
}
