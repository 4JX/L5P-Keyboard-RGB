use super::color_tiles::ColorTilesState;
use crate::enums::Effects;

pub struct Profile {
	pub color_tiles_state: ColorTilesState,
	pub effect: Effects,
	pub speed: u8,
	pub brightness: u8,
}
