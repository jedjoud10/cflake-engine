#![warn(missing_docs)]

//! TODO: Docs

/// Main events that we ought to hook onto
pub mod events;

/// Plugin system
pub mod plugin;

/// Resource management
pub mod resource;

/// System execution and scheduling
pub mod system;
mod tests;

/// Main world that contains the resources
pub mod world;

/// Re-exports everything
pub mod prelude {
    pub use crate::events::*;
    pub use crate::plugin::*;
    pub use crate::resource::*;
    pub use crate::system::*;
    pub use crate::world::*;
}

pub use prelude::*;