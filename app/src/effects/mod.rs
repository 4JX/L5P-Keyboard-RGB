use crate::{
	enums::{Direction, Effects, Message},
	profile::{self, Profile},
};

use crossbeam_channel::{Receiver, Sender};
use error_stack::{IntoReport, Result, ResultExt};
use legion_rgb_driver::{BaseEffects, Keyboard};
use rand::thread_rng;
use std::{
	sync::atomic::{AtomicBool, Ordering},
	thread,
	time::Duration,
};
use std::{sync::Arc, thread::JoinHandle};
use thiserror::Error;

use self::{
	ambient::AmbientLight,
	christmas::Christmas,
	custom_effect::{CustomEffect, EffectType},
	disco::Disco,
	fade::Fade,
	lightning::Lightning,
	ripple::Ripple,
	swipe::Swipe,
	temperature::Temperature,
};

mod ambient;
mod christmas;
pub mod custom_effect;
mod disco;
mod fade;
mod lightning;
mod ripple;
mod swipe;
mod temperature;

#[derive(Debug, Error)]
#[error("There was an error getting a valid keyboard")]
pub struct AcquireKeyboardError;

#[derive(Debug, Error)]
#[error("Could not create keyboard manager")]
pub struct ManagerCreationError;

/// Manager wrapper
pub struct EffectManager {
	pub tx: Sender<Message>,
	inner_handle: JoinHandle<()>,
	stop_signals: StopSignals,
}

/// Controls the keyboard lighting logic
struct Inner {
	keyboard: Keyboard,
	tx: Sender<Message>,
	rx: Receiver<Message>,
	stop_signals: StopSignals,
	last_profile: Profile,
}

pub enum OperationMode {
	Cli,
	Gui,
}

impl EffectManager {
	pub fn new(operation_mode: OperationMode) -> Result<Self, ManagerCreationError> {
		let stop_signals = StopSignals {
			manager_stop_signal: Arc::new(AtomicBool::new(false)),
			keyboard_stop_signal: Arc::new(AtomicBool::new(false)),
		};

		let mut keyboard_result = legion_rgb_driver::get_keyboard(stop_signals.keyboard_stop_signal.clone())
			.into_report()
			.change_context(AcquireKeyboardError)
			.attach_printable("Ensure that you have a supported model and that the application has access to it.");

		#[cfg(target_os = "linux")]
		{
			keyboard_result = keyboard_result.attach_printable("On Linux, see https://github.com/4JX/L5P-Keyboard-RGB#usage");
		}

		let keyboard = keyboard_result.change_context(ManagerCreationError)?;

		let (tx, rx) = crossbeam_channel::unbounded::<Message>();

		let mut inner = Inner {
			keyboard,
			rx,
			tx: tx.clone(),
			stop_signals: stop_signals.clone(),
			last_profile: Profile::default(),
		};

		macro_rules! effect_thread_loop {
			($e: expr) => {
				thread::spawn(move || loop {
					match $e {
						Some(message) => match message {
							Message::Refresh => {
								inner.refresh();
							}
							Message::Profile { profile } => {
								inner.set_profile(profile);
							}
							Message::CustomEffect { effect } => {
								inner.custom_effect(effect);
							}
							Message::Exit => break,
						},
						None => {
							thread::sleep(Duration::from_millis(20));
						}
					}
				})
			};
		}

		let inner_handle = match operation_mode {
			OperationMode::Cli => effect_thread_loop!(inner.rx.try_recv().ok()),
			OperationMode::Gui => effect_thread_loop!(inner.rx.try_iter().last()),
		};

		let manager = Self { tx, inner_handle, stop_signals };

		Ok(manager)
	}

	pub fn set_profile(&mut self, profile: Profile) {
		self.stop_signals.store_true();
		self.tx.try_send(Message::Profile { profile }).unwrap();
	}

	pub fn custom_effect(&self, effect: CustomEffect) {
		self.stop_signals.store_true();
		self.tx.send(Message::CustomEffect { effect }).unwrap();
	}

	pub fn join_and_exit(self) {
		self.tx.send(Message::Exit).unwrap();
		self.inner_handle.join().unwrap();
	}
}

impl Inner {
	fn refresh(&mut self) {
		self.set_profile(self.last_profile.clone());
	}

	fn set_profile(&mut self, mut profile: Profile) {
		self.last_profile = profile.clone();

		self.stop_signals.store_false();
		let mut thread_rng = thread_rng();

		self.keyboard.set_effect(BaseEffects::Static).unwrap();
		if profile.effect.is_built_in() {
			self.keyboard.set_speed(profile.speed).unwrap();
		};
		self.keyboard.set_brightness(profile.brightness).unwrap();

		match profile.effect {
			Effects::Static => {
				self.keyboard.set_colors_to(&profile.rgb_array()).unwrap();
				self.keyboard.set_effect(BaseEffects::Static).unwrap();
			}
			Effects::Breath => {
				self.keyboard.set_colors_to(&profile.rgb_array()).unwrap();
				self.keyboard.set_effect(BaseEffects::Breath).unwrap();
			}
			Effects::Smooth => {
				self.keyboard.set_effect(BaseEffects::Smooth).unwrap();
			}
			Effects::Wave => match profile.direction {
				Direction::Left => {
					self.keyboard.set_effect(BaseEffects::LeftWave).unwrap();
				}
				Direction::Right => {
					self.keyboard.set_effect(BaseEffects::RightWave).unwrap();
				}
			},

			Effects::Lightning => Lightning::play(self, &profile, &mut thread_rng),
			Effects::AmbientLight { mut fps } => {
				fps = fps.clamp(1, 60);

				AmbientLight::play(self, fps)
			}
			Effects::SmoothWave => {
				profile.rgb_zones = profile::arr_to_zones([255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 255]);
				Swipe::play(self, &profile)
			}
			Effects::Swipe => Swipe::play(self, &profile),
			Effects::Disco => Disco::play(self, &profile, &mut thread_rng),
			Effects::Christmas => Christmas::play(self, &mut thread_rng),
			Effects::Fade => Fade::play(self, &profile),
			Effects::Temperature => Temperature::play(self),
			Effects::Ripple => Ripple::play(self, &profile),
		}
		self.stop_signals.store_false();
	}

	fn custom_effect(&mut self, custom_effect: CustomEffect) {
		self.stop_signals.store_false();

		'outer: loop {
			for step in custom_effect.effect_steps.clone() {
				self.keyboard.set_speed(step.speed).unwrap();
				self.keyboard.set_brightness(step.brightness).unwrap();
				if let EffectType::Set = step.step_type {
					self.keyboard.set_colors_to(&step.rgb_array).unwrap();
				} else {
					self.keyboard.transition_colors_to(&step.rgb_array, step.steps, step.delay_between_steps).unwrap();
				}
				if self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
					break 'outer;
				}
				thread::sleep(Duration::from_millis(step.sleep));
			}
			if !custom_effect.should_loop {
				break;
			}
		}
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
