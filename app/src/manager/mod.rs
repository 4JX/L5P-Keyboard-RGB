use crate::enums::{Direction, Effects, Message};

use crossbeam_channel::{Receiver, Sender};
use effects::{ambient, christmas, disco, fade, lightning, ripple, swipe, temperature};
use error_stack::{Result, ResultExt};
use legion_rgb_driver::{BaseEffects, Keyboard, SPEED_RANGE};
use profile::Profile;
use rand::{rng, rngs::ThreadRng};
use single_instance::SingleInstance;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};
use std::{sync::Arc, thread::JoinHandle};
use thiserror::Error;

use self::custom_effect::{CustomEffect, EffectType};

pub mod custom_effect;
mod effects;
pub mod profile;

pub use effects::show_effect_ui;

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
            .attach_printable("On Linux, you may need to configure additional permissions")
            .attach_printable("https://github.com/4JX/L5P-Keyboard-RGB#usage")?;

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
        self.stop_signals.store_true();
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
        let mut rng = rng();

        if profile.effect.is_built_in() {
            let clamped_speed = self.clamp_speed(profile.speed);
            self.keyboard.set_speed(clamped_speed).unwrap();
        } else {
            // All custom effects rely on rapidly switching a static color
            self.keyboard.set_effect(BaseEffects::Static).unwrap();
        }

        self.keyboard.set_brightness(profile.brightness as u8 + 1).unwrap();

        self.apply_effect(&mut profile, &mut rng);
        self.stop_signals.store_false();
    }

    fn clamp_speed(&self, speed: u8) -> u8 {
        speed.clamp(SPEED_RANGE.min().unwrap(), SPEED_RANGE.max().unwrap())
    }

    fn apply_effect(&mut self, profile: &mut Profile, rng: &mut ThreadRng) {
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
            Effects::Wave => {
                let effect = match profile.direction {
                    Direction::Left => BaseEffects::LeftWave,
                    Direction::Right => BaseEffects::RightWave,
                };
                self.keyboard.set_effect(effect).unwrap();
            }
            Effects::Lightning => lightning::play(self, profile, rng),
            Effects::AmbientLight { mut fps, mut saturation_boost } => {
                fps = fps.clamp(1, 60);
                saturation_boost = saturation_boost.clamp(0.0, 1.0);
                ambient::play(self, fps, saturation_boost);
            }
            Effects::SmoothWave { mode, clean_with_black } => {
                profile.rgb_zones = profile::arr_to_zones([255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 255]);
                swipe::play(self, profile, mode, clean_with_black);
            }
            Effects::Swipe { mode, clean_with_black } => swipe::play(self, profile, mode, clean_with_black),
            Effects::Disco => disco::play(self, profile, rng),
            Effects::Christmas => christmas::play(self, rng),
            Effects::Fade => fade::play(self, profile),
            Effects::Temperature => temperature::play(self),
            Effects::Ripple => ripple::play(self, profile),
        }
    }

    fn custom_effect(&mut self, custom_effect: &CustomEffect) {
        self.stop_signals.store_false();

        loop {
            for step in &custom_effect.effect_steps {
                self.keyboard.set_brightness(step.brightness).unwrap();
                match step.step_type {
                    EffectType::Set => {
                        self.keyboard.set_colors_to(&step.rgb_array).unwrap();
                    }
                    _ => {
                        self.keyboard.transition_colors_to(&step.rgb_array, step.steps, step.delay_between_steps).unwrap();
                    }
                }
                if self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
                    return;
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
