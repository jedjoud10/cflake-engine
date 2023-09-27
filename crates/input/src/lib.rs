#![warn(missing_docs)]

//! Input handling and mapping to user bindings

/// Mouse axii, gamepad axii
pub mod axis;

/// Mouse buttons, keyboard buttons, gamepad buttons
pub mod button;

/// Identifier traits
pub mod ids;

/// TODO
pub mod system;

/// Main module
pub mod input;

/// Re-exports everything
pub mod prelude {
    pub use crate::axis::*;
    pub use crate::button::*;
    pub use crate::ids::*;
    pub use crate::input::*;
    pub use crate::system::*;
}
