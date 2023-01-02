use crate::enums::{Direction, Effects};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Default, Debug)]
pub struct Profile {
	pub rgb_array: [u8; 12],
	pub effect: Effects,
	pub direction: Direction,
	pub speed: u8,
	pub brightness: u8,
	pub ui_toggle_button_state: [bool; 5],
}
