use crate::{
	enums::{Direction, Effects, Message},
	keyboard_utils,
};
use crate::{
	error,
	keyboard_utils::{BaseEffects, Keyboard},
};

use flume::{Receiver, Sender};
use rand::{rngs::ThreadRng, thread_rng};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use self::{ambient::AmbientLight, christmas::Christmas, disco::Disco, fade::Fade, lightning::Lightning, ripple::Ripple, smooth_wave::SmoothWave, swipe::Swipe, temperature::Temperature};

mod ambient;
mod christmas;
mod disco;
mod fade;
mod lightning;
mod ripple;
mod smooth_wave;
mod swipe;
mod temperature;

pub struct EffectManager {
	pub keyboard: Keyboard,
	pub rx: Receiver<Message>,
	pub tx: Sender<Message>,
	pub stop_signals: StopSignals,
	pub last_effect: Effects,
}

impl EffectManager {
	pub fn new() -> Result<Self, error::Error> {
		let keyboard_stop_signal = Arc::new(AtomicBool::new(false));
		let keyboard = keyboard_utils::get_keyboard(keyboard_stop_signal.clone())?;

		let (tx, rx) = flume::unbounded::<Message>();

		let manager = Self {
			keyboard,
			rx,
			tx,
			stop_signals: StopSignals {
				manager_stop_signal: Arc::new(AtomicBool::new(false)),
				keyboard_stop_signal,
			},
			last_effect: Effects::Static,
		};

		Ok(manager)
	}

	pub fn set_effect(&mut self, effect: Effects, direction: Direction, rgb_array: &[u8; 12], speed: u8, brightness: u8) {
		self.stop_signals.store_false();
		self.last_effect = effect;
		let mut thread_rng = thread_rng();

		self.keyboard.set_effect(BaseEffects::Static);
		self.keyboard.set_speed(speed);
		self.keyboard.set_brightness(brightness);

		match effect {
			Effects::Static => {
				self.keyboard.set_colors_to(rgb_array);
				self.keyboard.set_effect(BaseEffects::Static);
			}
			Effects::Breath => {
				self.keyboard.set_colors_to(rgb_array);
				self.keyboard.set_effect(BaseEffects::Breath);
			}
			Effects::Smooth => {
				self.keyboard.set_effect(BaseEffects::Smooth);
			}
			Effects::Wave => match direction {
				Direction::Left => self.keyboard.set_effect(BaseEffects::LeftWave),
				Direction::Right => self.keyboard.set_effect(BaseEffects::RightWave),
			},

			Effects::Lightning => Lightning::play(self, direction, rgb_array, speed, brightness, &mut thread_rng),
			Effects::AmbientLight => AmbientLight::play(self, direction, rgb_array, speed, brightness, &mut thread_rng),
			Effects::SmoothWave => SmoothWave::play(self, direction, rgb_array, speed, brightness, &mut thread_rng),
			Effects::Swipe => Swipe::play(self, direction, rgb_array, speed, brightness, &mut thread_rng),
			Effects::Disco => Disco::play(self, direction, rgb_array, speed, brightness, &mut thread_rng),
			Effects::Christmas => Christmas::play(self, direction, rgb_array, speed, brightness, &mut thread_rng),
			Effects::Fade => Fade::play(self, direction, rgb_array, speed, brightness, &mut thread_rng),
			Effects::Temperature => Temperature::play(self, direction, rgb_array, speed, brightness, &mut thread_rng),
			Effects::Ripple => Ripple::play(self, direction, rgb_array, speed, brightness, &mut thread_rng),
		}
		self.stop_signals.store_false();
	}
}

#[derive(Clone)]
pub struct StopSignals {
	pub manager_stop_signal: Arc<AtomicBool>,
	pub keyboard_stop_signal: Arc<AtomicBool>,
}

impl StopSignals {
	pub fn store_true(&self) {
		self.keyboard_stop_signal.store(true, Ordering::SeqCst);
		self.manager_stop_signal.store(true, Ordering::SeqCst);
	}
	pub fn store_false(&self) {
		self.keyboard_stop_signal.store(false, Ordering::SeqCst);
		self.manager_stop_signal.store(false, Ordering::SeqCst);
	}
}

trait EffectPlayer {
	fn play(manager: &mut EffectManager, direction: Direction, rgb_array: &[u8; 12], speed: u8, brightness: u8, thread_rng: &mut ThreadRng);
}
