use std::sync::{
	atomic::{AtomicBool, Ordering},
	mpsc::Sender,
	Arc,
};

use fltk::{browser::HoldBrowser, menu::Choice};

use crate::enums::{Effects, Message};

use super::color_tiles::ColorTiles;

pub struct App {
	pub color_tiles: ColorTiles,
	pub effect_browser: HoldBrowser,
	pub speed_choice: Choice,
	pub brightness_choice: Choice,
	pub tx: Sender<Message>,
	pub stop_signal: Arc<AtomicBool>,
}

impl App {
	pub fn set_effect(&mut self, effect: Effects) {
		match effect {
			Effects::Static => {
				self.color_tiles.activate();
			}
			Effects::Breath => {
				self.color_tiles.activate();
			}
			Effects::Smooth => {
				self.color_tiles.deactivate();
			}
			Effects::LeftWave => {
				self.color_tiles.deactivate();
			}
			Effects::RightWave => {
				self.color_tiles.deactivate();
			}
			Effects::Lightning => {
				self.color_tiles.deactivate();
			}
			Effects::AmbientLight => {
				self.color_tiles.deactivate();
			}
			Effects::SmoothLeftWave => {
				self.color_tiles.deactivate();
			}
			Effects::SmoothRightWave => {
				self.color_tiles.deactivate();
			}
			Effects::LeftSwipe => {
				self.color_tiles.activate();
			}
			Effects::RightSwipe => {
				self.color_tiles.activate();
			}
			Effects::Disco => {
				self.color_tiles.deactivate();
			}
		}

		self.stop_signal.store(true, Ordering::SeqCst);
		self.tx.send(Message::UpdateEffect { effect }).unwrap();
	}
}
