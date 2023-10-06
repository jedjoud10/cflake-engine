#![allow(ambiguous_glob_reexports)]

pub mod app;
pub use ecs;
pub use graphics;
mod app_utils;
pub use input;
pub use utils;

/// Prelude that re-exports most of the types and interfaces used within cFlake engine
pub mod prelude {
    pub use crate::app::*;
    pub use crate::ecs::prelude::*;
    pub use crate::graphics::prelude::*;
    pub use crate::input::prelude::*;
    pub use crate::utils::prelude::*;
}
