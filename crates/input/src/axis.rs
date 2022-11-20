use serde::{Deserialize, Serialize};

// Convert a gilrs axis to an input axis
pub fn from_gilrs_axis(axis: gilrs::Axis) -> Option<Axis> {
    if matches!(axis, gilrs::Axis::Unknown) {
        return None;
    }

    unsafe {
        let mut code =
            std::mem::transmute::<gilrs::Axis, u16>(axis) as u32;
        code += OFFSET;
        Some(std::mem::transmute::<u32, crate::Axis>(code))
    }
}

// An axis can be mapped to a specific binding to be able to fetch it using a user defined name
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[repr(u32)]
pub enum Axis {
    // Mouse axii
    MousePositionX = 0,
    MousePositionY,
    MouseScroll,
    MousePositionDeltaX,
    MousePositionDeltaY,
    MouseScrollDelta,

    // Gamepad axii
    GamePadLeftStickX = AXIS_LSTICKX,
    GamePadLeftStickY = AXIS_LSTICKY,
    GamePadLeftZ = AXIS_LEFTZ,
    GamePadRightStickX = AXIS_RSTICKX,
    GamePadRightStickY = AXIS_RSTICKY,
    GamePadRightZ = AXIS_RIGHTZ,
    GamePadDPadX = AXIS_DPADX,
    GamePadDPadY = AXIS_DPADY,
}

// Axis code mappings copied from gilrs source code
pub const OFFSET: u32 = 5;
pub const AXIS_LSTICKX: u32 = 1 + OFFSET;
pub const AXIS_LSTICKY: u32 = 2 + OFFSET;
pub const AXIS_LEFTZ: u32 = 3 + OFFSET;
pub const AXIS_RSTICKX: u32 = 4 + OFFSET;
pub const AXIS_RSTICKY: u32 = 5 + OFFSET;
pub const AXIS_RIGHTZ: u32 = 6 + OFFSET;
pub const AXIS_DPADX: u32 = 7 + OFFSET;
pub const AXIS_DPADY: u32 = 8 + OFFSET;
