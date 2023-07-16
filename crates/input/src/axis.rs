use serde::{Deserialize, Serialize};

/// Mouse axis that we can bind to an axis mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseAxis {
    /// X position of the mouse
    PositionX,

    /// Y position of the mouse 
    PositionY,

    /// Current scroll value (integral of ScrollDelta)
    Scroll,

    /// Derivative of the X position of the mouse
    DeltaX,

    /// Derivative of the Y position of the mouse
    DeltaY,

    /// How much the scroll wheel scrolled
    ScrollDelta,
}

/// GilRS gamepad axis
pub type GamepadAxis = gilrs::Axis;

/// An axis can be mapped to a specific binding to be able to fetch it using a user defined name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum Axis {
    /// Mouse axii.
    Mouse(MouseAxis),

    /// Gamepad axii.
    Gamepad(GamepadAxis),
}

impl From<GamepadAxis> for Axis {
    fn from(value: GamepadAxis) -> Self {
        Axis::Gamepad(value)
    }
}

impl From<MouseAxis> for Axis {
    fn from(value: MouseAxis) -> Self {
        Axis::Mouse(value)
    }
}
