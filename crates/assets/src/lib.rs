#![warn(missing_docs)]

//! TODO: Docs

/// Module containing the [Asset] and [AsyncAsset] traits
pub mod asset;

/// Contains the singular error enum
pub mod error;

/// Handle fetching path, context, and settings from an input tuple
pub mod input;

/// Main asset loader; [Assets]
pub mod loader;

/// Macros for defining assets and their paths
pub mod macros;

/// Main asset loading system
pub mod plugin;

mod tests;

/// Re-exports everything
pub mod prelude {
    pub use crate::asset::*;
    pub use crate::error::*;
    pub use crate::input::*;
    pub use crate::loader::*;
    pub use crate::plugin::*;
    pub use crate::macros::asset;
}

pub use prelude::*;