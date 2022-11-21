use crate::{Axis, Button, ButtonState, Input};
use serde::*;

// Trait implemented for structs that allow us to fetch the key state from the main input handler
pub trait InputButtonId {
    fn get(self, input: &Input) -> ButtonState;
}

impl InputButtonId for Button {
    fn get(self, input: &Input) -> ButtonState {
        input.keys.get(&self).cloned().unwrap_or(ButtonState::None)
    }
}

impl InputButtonId for &'static str {
    fn get(self, input: &Input) -> ButtonState {
        input
            .bindings
            .key_bindings
            .get(self)
            .map(|key| Button::get(*key, input))
            .unwrap_or(ButtonState::None)
    }
}

// Trait implemented for structs that allow us to fetch the axis state from the main input handler
pub trait InputAxisId {
    fn get(self, input: &Input) -> f32;
}

impl InputAxisId for Axis {
    fn get(self, input: &Input) -> f32 {
        input.axii.get(&self).cloned().unwrap_or_default()
    }
}

impl InputAxisId for &'static str {
    fn get(self, input: &Input) -> f32 {
        input
            .bindings
            .axis_bindings
            .get(self)
            .map(|axis| Axis::get(*axis, input))
            .unwrap_or_default()
    }
}
