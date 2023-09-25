#![allow(ambiguous_glob_reexports)]

pub mod app;
mod logger;
pub use ecs;
pub use input;
pub use utils;

/// Prelude that re-exports most of the types and interfaces used within cFlake engine
pub mod prelude {
    pub use crate::app::*;
    pub use crate::ecs::*;
    pub use crate::input::*;
    pub use crate::utils::*;
}
