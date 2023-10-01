use serde::*;
use winit::event::ElementState;

/// The current state of any key / button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    /// The button just got pressed this frame.
    Pressed,

    /// The button was pressed/held last frame, but not this frame.
    Released,

    /// The button was kept pressed from last frame till this frame.
    Held,

    /// The button was not touched this frame nor last frame.
    None,
}

impl From<ElementState> for ButtonState {
    fn from(state: ElementState) -> Self {
        match state {
            ElementState::Pressed => Self::Pressed,
            ElementState::Released => Self::Released,
        }
    }
}

impl ButtonState {
    /// This checks if the state is equal to State::Pressed.
    pub fn pressed(&self) -> bool {
        matches!(self, ButtonState::Pressed)
    }

    /// This checks if the state is equal to State::Released.
    pub fn released(&self) -> bool {
        matches!(self, ButtonState::Released)
    }

    /// This checks if the State is equal to State::Held.
    pub fn held(&self) -> bool {
        matches!(self, ButtonState::Held)
    }
}

/// Keyboard button / key
pub type KeyboardButton = winit::keyboard::Key;

/// Mouse button / key
pub type MouseButton = winit::event::MouseButton;

/// Gamepad button / key
pub type GamepadButton = gilrs::Button;

/// A button that might come from the keyboard, mouse, or a gamepad
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
#[repr(u32)]
pub enum Button {
    /// Any sort of keyboard button
    Keyboard(KeyboardButton),

    /// Mouse buttons that we can press
    Mouse(MouseButton),

    /// Gamepad buttons
    Gamepad(GamepadButton),
}

impl From<KeyboardButton> for Button {
    fn from(value: KeyboardButton) -> Self {
        Button::Keyboard(value)
    }
}

impl From<MouseButton> for Button {
    fn from(value: MouseButton) -> Self {
        Button::Mouse(value)
    }
}

impl From<GamepadButton> for Button {
    fn from(value: GamepadButton) -> Self {
        Button::Gamepad(value)
    }
}
