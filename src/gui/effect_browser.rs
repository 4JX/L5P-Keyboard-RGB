use crate::enums::Effects;

use super::enums::Colors;
use fltk::{
	browser::HoldBrowser,
	enums::{Color, FrameType},
	group::Tile,
	prelude::*,
};
use strum::IntoEnumIterator;

pub struct EffectBrowser;

impl EffectBrowser {
	pub fn create() -> HoldBrowser {
		let mut effect_browser = HoldBrowser::new(0, 0, 310, 310, "").center_of_parent();
		for effect in Effects::iter() {
			effect_browser.add(effect.to_string().as_str());
		}

		effect_browser.set_frame(FrameType::RFlatBox);
		effect_browser.set_color(Color::from_u32(Colors::LighterGray as u32));
		effect_browser.set_selection_color(Color::from_u32(Colors::White as u32));
		effect_browser.set_text_size(20);
		effect_browser.select(1);
		effect_browser
	}
}

pub struct EffectBrowserTile {
	pub effect_browser: HoldBrowser,
}

impl EffectBrowserTile {
	pub fn create(x: i32, y: i32) -> Self {
		let mut effect_browser_tile = Tile::new(x, y, 360, 360, "");
		let effect_browser = EffectBrowser::create();
		effect_browser_tile.end();

		effect_browser_tile.set_frame(FrameType::FlatBox);
		effect_browser_tile.set_color(Color::from_u32(Colors::DarkerGray as u32));

		Self { effect_browser }
	}
}
