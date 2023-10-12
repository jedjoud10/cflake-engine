#![warn(missing_docs)]

//! Graphics wrapper around phobos for quality of life improvements like type safe buffers and typed texture formats

/// Contains core information and resources used for context initialization
pub mod context;

/// Main graphics plugin that will register the phobos context
pub mod plugin;

/// Re-exports everything
pub mod prelude {
    pub use crate::context::*;
}

pub use phobos;
pub use prelude::*;