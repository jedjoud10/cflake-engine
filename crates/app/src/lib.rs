#![allow(ambiguous_glob_reexports)]

pub mod app;
pub use ecs;
pub use rendering;
mod app_utils;
pub use input;
pub use utils;
pub use world;

/// Prelude that re-exports most of the types and interfaces used within cFlake engine
pub mod prelude {
    pub use crate::world::*;
    pub use crate::app::*;
    pub use crate::ecs::*;
    pub use crate::rendering::*;
    pub use crate::input::*;
    pub use crate::utils::*;
}
