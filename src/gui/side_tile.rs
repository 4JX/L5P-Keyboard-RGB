use crate::{
	effects::StopSignals,
	enums::{Effects, Message},
};

use super::enums::Colors;
use fltk::{
	browser::HoldBrowser,
	button::{Button, RadioButton},
	enums::{Color, FrameType},
	group::Tile,
	prelude::*,
};
use strum::IntoEnumIterator;

const TILE_WIDTH: i32 = 360;
const TILE_HEIGHT: i32 = 450;

pub struct SideTile {
	pub effect_browser: HoldBrowser,
	pub profile_browser: HoldBrowser,
	pub add_profile_button: Button,
	pub remove_profile_button: Button,
}

impl SideTile {
	pub fn create(x: i32, y: i32, tx: &flume::Sender<Message>, stop_signals: &StopSignals) -> Self {
		let mut exterior_tile = Tile::new(x, y, TILE_WIDTH, TILE_HEIGHT, "");

		let padding = 50;
		let button_width = TILE_WIDTH / 2 - padding / 2;
		let button_height = 40;

		let browser_buttons_tile = Tile::new(x, y + padding / 2, TILE_WIDTH, button_height, "");
		let mut effect_browser_button = RadioButton::new(x + padding / 2, 0, button_width, button_height, "Effect Browser").center_y(&browser_buttons_tile);
		let mut profile_browser_button = RadioButton::new(x + TILE_WIDTH / 2, 0, button_width, button_height, "Profiles").center_y(&browser_buttons_tile);
		browser_buttons_tile.end();

		effect_browser_button.set_label_color(Color::from_u32(Colors::LightGray as u32));
		effect_browser_button.set_frame(FrameType::FlatBox);
		effect_browser_button.toggle(true);

		profile_browser_button.set_label_color(Color::from_u32(Colors::LightGray as u32));
		profile_browser_button.set_frame(FrameType::FlatBox);

		let lower_tile_y_change = button_height + padding / 2;
		let lower_tile_height = TILE_HEIGHT - lower_tile_y_change;
		let lower_tile = Tile::new(x, y + lower_tile_y_change, TILE_WIDTH, lower_tile_height, "");
		let mut effect_browser = HoldBrowser::new(0, 0, TILE_WIDTH - padding, lower_tile_height - padding, "").center_of_parent();
		let mut profile_browser = HoldBrowser::new(20, 0, TILE_WIDTH - padding, lower_tile_height - padding - button_height, "")
			.above_of(&lower_tile, -(lower_tile_height - padding / 2 - button_height))
			.center_x(&lower_tile);

		let mut profile_buttons_tile = Tile::new(0, y + padding + (lower_tile_height - padding), TILE_WIDTH, button_height, "").center_x(&lower_tile);
		let mut add_profile_button = Button::new(x + padding / 2, 0, button_width, button_height, "+").center_y(&profile_buttons_tile);
		let mut remove_profile_button = Button::new(x + TILE_WIDTH / 2, 0, button_width, button_height, "-").center_y(&profile_buttons_tile);
		profile_buttons_tile.end();

		add_profile_button.set_label_size(24);
		add_profile_button.set_label_color(Color::from_u32(Colors::DarkerGray as u32));
		add_profile_button.set_frame(FrameType::FlatBox);

		remove_profile_button.set_label_size(24);
		remove_profile_button.set_label_color(Color::from_u32(Colors::DarkerGray as u32));
		remove_profile_button.set_frame(FrameType::FlatBox);

		lower_tile.end();

		profile_browser.hide();
		profile_buttons_tile.hide();

		effect_browser_button.set_callback({
			let mut effect_browser = effect_browser.clone();
			let mut profile_browser = profile_browser.clone();
			let mut profile_buttons_tile = profile_buttons_tile.clone();
			move |_button| {
				effect_browser.show();
				profile_browser.hide();
				profile_buttons_tile.hide();
			}
		});

		profile_browser_button.set_callback({
			let mut effect_browser = effect_browser.clone();
			let mut profile_browser = profile_browser.clone();
			let mut profile_buttons_tile = profile_buttons_tile.clone();
			move |_button| {
				effect_browser.hide();
				profile_browser.show();
				profile_buttons_tile.show();
			}
		});

		for effect in Effects::iter() {
			effect_browser.add(effect.to_string().as_str());
		}

		effect_browser.set_frame(FrameType::RFlatBox);
		effect_browser.set_color(Color::from_u32(Colors::LighterGray as u32));
		effect_browser.set_selection_color(Color::from_u32(Colors::White as u32));
		effect_browser.set_text_size(20);
		effect_browser.select(1);

		effect_browser.set_callback({
			let stop_signals = stop_signals.clone();
			let tx = tx.clone();
			move |_browser| {
				stop_signals.store_true();
				tx.send(Message::Refresh).unwrap();
			}
		});

		profile_browser.set_frame(FrameType::RFlatBox);
		profile_browser.set_color(Color::from_u32(Colors::LighterGray as u32));
		profile_browser.set_selection_color(Color::from_u32(Colors::White as u32));
		profile_browser.set_text_size(20);
		profile_browser.select(1);

		exterior_tile.end();

		exterior_tile.set_frame(FrameType::FlatBox);
		exterior_tile.set_color(Color::from_u32(Colors::DarkerGray as u32));

		Self {
			effect_browser,
			profile_browser,
			add_profile_button,
			remove_profile_button,
		}
	}
}
