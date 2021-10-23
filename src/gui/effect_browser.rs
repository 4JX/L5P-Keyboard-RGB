use fltk::{
	browser::HoldBrowser,
	enums::{Color, FrameType},
	group::Tile,
	prelude::*,
};

const WHITE: u32 = 0xffffff;
const LIGHTER_GRAY: u32 = 0xcccccc;

pub struct EffectBrowser;

impl EffectBrowser {
	pub fn new(effects_list: &Vec<&str>) -> HoldBrowser {
		let mut effect_type_tile = Tile::new(540, 0, 360, 360, "");
		let mut effect_browser = HoldBrowser::new(0, 0, 310, 310, "").center_of_parent();
		for effect in effects_list.iter() {
			effect_browser.add(effect);
		}
		effect_type_tile.end();

		effect_type_tile.set_frame(FrameType::FlatBox);
		effect_type_tile.set_color(Color::from_u32(0x222222));

		// Effect choice
		effect_browser.set_frame(FrameType::FlatBox);
		effect_browser.set_color(Color::from_u32(LIGHTER_GRAY));
		effect_browser.set_selection_color(Color::from_u32(WHITE));
		effect_browser.set_text_size(20);
		effect_browser.select(1);
		effect_browser
	}
}
