use crate::{Axis, Button, ButtonState, Input};

/// Trait implemented for structs that allow us to fetch the key state from the main input handler.
/// Allows for use to pass `&str` and `Button` interchangeably into methods that require fetching button state.
pub trait InputButtonId {
    /// Get the button state using `self` as an identifier
    fn get(self, input: &Input) -> ButtonState;
}

impl<T: Into<Button>> InputButtonId for T {
    fn get(self, input: &Input) -> ButtonState {
        let converted = self.into();
        input
            .keys
            .get(&converted)
            .cloned()
            .unwrap_or(ButtonState::None)
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

/// Trait implemented for structs that allow us to fetch the axis state from the main input handler.
/// Allows for use to pass `&str` and `Axis` interchangeably into methods that require fetching button state.
pub trait InputAxisId {
    /// Get the input state using `self` as an identifier
    fn get(self, input: &Input) -> f32;
}

impl<T: Into<Axis>> InputAxisId for T {
    fn get(self, input: &Input) -> f32 {
        let converted = self.into();
        input.axii.get(&converted).cloned().unwrap_or_default()
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
