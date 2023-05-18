use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseAxis {
    PositionX,
    PositionY,
    Scroll,
    DeltaX,
    DeltaY,
    ScrollDelta,
}

pub type GamepadAxis = gilrs::Axis;

// An axis can be mapped to a specific binding to be able to fetch it using a user defined name
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum Axis {
    // Mouse axii
    Mouse(MouseAxis),

    // Gamepad axii
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
