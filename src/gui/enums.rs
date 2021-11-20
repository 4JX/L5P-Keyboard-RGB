#[derive(Clone, Copy)]
pub enum BaseColor {
	Red,
	Green,
	Blue,
}

pub enum Colors {
	Red = 0xff0000,
	DarkRed = 0x893838,
	Green = 0x00ff00,
	Blue = 0x0000ff,
	DarkerGray = 0x222222,
	DarkGray = 0x333333,
	Gray = 0x444444,
	LightGray = 0x777777,
	LighterGray = 0xcccccc,
	White = 0xeeeeee,
}
