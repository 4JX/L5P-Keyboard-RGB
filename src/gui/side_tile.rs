use crate::{
	enums::{Effects, Message},
	keyboard_manager::StopSignals,
};

use super::enums::Colors;
use fltk::{
	browser::HoldBrowser,
	button::RadioButton,
	enums::{Color, FrameType},
	group::Tile,
	prelude::*,
};
use strum::IntoEnumIterator;

const TILE_WIDTH: i32 = 360;
const TILE_HEIGHT: i32 = 450;

pub struct SideTile {
	pub effect_browser: HoldBrowser,
}

impl SideTile {
	pub fn create(x: i32, y: i32, tx: &flume::Sender<Message>, stop_signals: &StopSignals) -> Self {
		let mut exterior_tile = Tile::new(x, y, TILE_WIDTH, TILE_HEIGHT, "");

		let padding = 50;
		let button_width = TILE_WIDTH / 2 - padding / 2;
		let button_height = 40;

		let buttons_tile = Tile::new(x, y + padding / 2, TILE_WIDTH, button_height, "");
		let mut effect_browser_button = RadioButton::new(x + padding / 2, 0, button_width, button_height, "Effect Browser").center_y(&buttons_tile);
		let mut presets_button = RadioButton::new(x + TILE_WIDTH / 2, 0, button_width, button_height, "Presets").center_y(&buttons_tile);
		buttons_tile.end();

		let effect_browser_tile_y_change = button_height + padding / 2;
		let effect_browser_tile_height = TILE_HEIGHT - effect_browser_tile_y_change;
		let effect_browser_tile = Tile::new(x, y + effect_browser_tile_y_change, TILE_WIDTH, effect_browser_tile_height, "");
		let mut effect_browser = HoldBrowser::new(0, 0, TILE_WIDTH - padding, effect_browser_tile_height - padding, "").center_of_parent();
		effect_browser_tile.end();

		effect_browser_button.set_callback({
			let mut effect_browser = effect_browser.clone();
			move |_button| {
				effect_browser.show();
			}
		});

		presets_button.set_callback({
			let mut effect_browser = effect_browser.clone();
			move |_button| {
				effect_browser.hide();
			}
		});

		for effect in Effects::iter() {
			#[cfg(target_os = "windows")]
			if effect == Effects::Temperature {
				continue;
			}
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
			move |browser| {
				stop_signals.store_true();
				match browser.value() {
					0 => {
						browser.select(0);
					}
					1 => {
						tx.send(Message::UpdateEffect { effect: Effects::Static }).unwrap();
					}
					2 => {
						tx.send(Message::UpdateEffect { effect: Effects::Breath }).unwrap();
					}
					3 => {
						tx.send(Message::UpdateEffect { effect: Effects::Smooth }).unwrap();
					}
					4 => {
						tx.send(Message::UpdateEffect { effect: Effects::Wave }).unwrap();
					}
					5 => {
						tx.send(Message::UpdateEffect { effect: Effects::Lightning }).unwrap();
					}
					6 => {
						tx.send(Message::UpdateEffect { effect: Effects::AmbientLight }).unwrap();
					}
					7 => {
						tx.send(Message::UpdateEffect { effect: Effects::SmoothWave }).unwrap();
					}
					8 => {
						tx.send(Message::UpdateEffect { effect: Effects::Swipe }).unwrap();
					}
					9 => {
						tx.send(Message::UpdateEffect { effect: Effects::Disco }).unwrap();
					}
					10 => {
						tx.send(Message::UpdateEffect { effect: Effects::Christmas }).unwrap();
					}
					11 => {
						tx.send(Message::UpdateEffect { effect: Effects::Fade }).unwrap();
					}
					12 => {
						tx.send(Message::UpdateEffect { effect: Effects::Temperature }).unwrap();
					}
					_ => unreachable!("Effect index is out of range"),
				}
			}
		});

		exterior_tile.end();

		exterior_tile.set_frame(FrameType::FlatBox);
		exterior_tile.set_color(Color::from_u32(Colors::DarkerGray as u32));

		Self { effect_browser }
	}
}
