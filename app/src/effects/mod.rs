use crate::{
    enums::{Direction, Effects, Message},
    profile::{self, Profile},
};

use crossbeam_channel::{Receiver, Sender};
use error_stack::{Result, ResultExt};
use legion_rgb_driver::{BaseEffects, Keyboard, SPEED_RANGE};
use rand::thread_rng;
use single_instance::SingleInstance;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};
use std::{sync::Arc, thread::JoinHandle};
use thiserror::Error;

use self::custom_effect::{CustomEffect, EffectType};

mod ambient;
mod christmas;
pub mod custom_effect;
mod disco;
mod fade;
mod lightning;
mod ripple;
mod swipe;
mod temperature;
mod zones;

#[derive(Debug, Error, PartialEq)]
#[error("Could not create keyboard manager")]
pub enum ManagerCreationError {
    #[error("There was an error getting a valid keyboard")]
    AcquireKeyboard,
    #[error("An instance of the program is already running")]
    InstanceAlreadyRunning,
}

/// Manager wrapper
pub struct EffectManager {
    pub tx: Sender<Message>,
    inner_handle: Option<JoinHandle<()>>,
    stop_signals: StopSignals,
}

/// Controls the keyboard lighting logic
struct Inner {
    keyboard: Keyboard,
    rx: Receiver<Message>,
    stop_signals: StopSignals,
    last_profile: Profile,
    // Can't drop this else it stops "reserving" whatever underlying implementation identifier it uses
    #[allow(dead_code)]
    single_instance: SingleInstance,
}

#[derive(Clone, Copy)]
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

        // Use the crate's name as the identifier, should be unique enough
        let single_instance = SingleInstance::new(env!("CARGO_PKG_NAME")).unwrap();

        if !single_instance.is_single() {
            return Err(ManagerCreationError::InstanceAlreadyRunning.into());
        }

        let keyboard = legion_rgb_driver::get_keyboard(stop_signals.keyboard_stop_signal.clone())
            .change_context(ManagerCreationError::AcquireKeyboard)
            .attach_printable("Ensure that you have a supported model and that the application has access to it.")
            .attach_printable("On Linux, see https://github.com/4JX/L5P-Keyboard-RGB#usage")?;

        let (tx, rx) = crossbeam_channel::unbounded::<Message>();

        let mut inner = Inner {
            keyboard,
            rx,
            stop_signals: stop_signals.clone(),
            last_profile: Profile::default(),
            single_instance,
        };

        macro_rules! effect_thread_loop {
            ($e: expr) => {
                thread::spawn(move || loop {
                    match $e {
                        Some(message) => match message {
                            Message::Profile { profile } => {
                                inner.set_profile(profile);
                            }
                            Message::CustomEffect { effect } => {
                                inner.custom_effect(&effect);
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

        let manager = Self {
            tx,
            inner_handle: Some(inner_handle),
            stop_signals,
        };

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

    pub fn shutdown(mut self) {
        self.tx.send(Message::Exit).unwrap();
        if let Some(handle) = self.inner_handle.take() {
            handle.join().unwrap();
        };
    }
}

impl Inner {
    fn set_profile(&mut self, mut profile: Profile) {
        self.last_profile = profile.clone();

        self.stop_signals.store_false();
        let mut thread_rng = thread_rng();

        self.keyboard.set_effect(BaseEffects::Static).unwrap();
        if profile.effect.is_built_in() {
            let clamped_speed = profile.speed.clamp(SPEED_RANGE.min().unwrap(), SPEED_RANGE.max().unwrap());

            self.keyboard.set_speed(clamped_speed).unwrap();
        };
        self.keyboard.set_brightness(profile.brightness as u8 + 1).unwrap();

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

            Effects::Lightning => lightning::play(self, &profile, &mut thread_rng),
            Effects::AmbientLight { mut fps, mut saturation_boost } => {
                fps = fps.clamp(1, 60);
                saturation_boost = saturation_boost.clamp(0.0, 1.0);

                ambient::play(self, fps, saturation_boost);
            }
            Effects::SmoothWave => {
                profile.rgb_zones = profile::arr_to_zones([255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 255]);
                swipe::play(self, &profile);
            }
            Effects::Swipe => swipe::play(self, &profile),
            Effects::Disco => disco::play(self, &profile, &mut thread_rng),
            Effects::Christmas => christmas::play(self, &mut thread_rng),
            Effects::Fade => fade::play(self, &profile),
            Effects::Temperature => temperature::play(self),
            Effects::Ripple => ripple::play(self, &profile),
        }
        self.stop_signals.store_false();
    }

    fn custom_effect(&mut self, custom_effect: &CustomEffect) {
        self.stop_signals.store_false();

        'outer: loop {
            for step in custom_effect.effect_steps.clone() {
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

impl Drop for EffectManager {
    fn drop(&mut self) {
        let _ = self.tx.send(Message::Exit);
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
