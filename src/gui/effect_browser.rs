use crate::{
	enums::{Effects, Message},
	keyboard_manager::StopSignals,
};

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
	pub fn create(tx: &flume::Sender<Message>, stop_signals: &StopSignals) -> HoldBrowser {
		let mut effect_browser = HoldBrowser::new(0, 0, 310, 400, "").center_of_parent();
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

		effect_browser
	}
}

pub struct EffectBrowserTile {
	pub effect_browser: HoldBrowser,
}

impl EffectBrowserTile {
	pub fn create(x: i32, y: i32, tx: &flume::Sender<Message>, stop_signals: &StopSignals) -> Self {
		let mut effect_browser_tile = Tile::new(x, y, 360, 450, "");
		let effect_browser = EffectBrowser::create(tx, stop_signals);
		effect_browser_tile.end();

		effect_browser_tile.set_frame(FrameType::FlatBox);
		effect_browser_tile.set_color(Color::from_u32(Colors::DarkerGray as u32));

		Self { effect_browser }
	}
}
