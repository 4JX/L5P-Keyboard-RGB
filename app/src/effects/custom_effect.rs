use std::path::Path;

use error_stack::{Result, ResultExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::util::StorageTrait;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct EffectStep {
    pub rgb_array: [u8; 12],
    pub step_type: EffectType,
    pub brightness: u8,
    pub steps: u8,
    pub delay_between_steps: u64,
    pub sleep: u64,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum EffectType {
    Set,
    Transition,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CustomEffect {
    pub effect_steps: Vec<EffectStep>,
    pub should_loop: bool,
}

#[derive(Debug, Error)]
#[error("Could not load custom effect")]
pub struct LoadCustomEffectError;

impl CustomEffect {
    pub fn from_file(path: &Path) -> Result<Self, LoadCustomEffectError> {
        Self::load(path).change_context(LoadCustomEffectError)
    }
}

impl<'a> StorageTrait<'a> for CustomEffect {}
