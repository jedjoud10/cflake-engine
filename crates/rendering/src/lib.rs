#![warn(missing_docs)]

//! Main rendering crate that creates some QoL wrapeprs around phobos types
//! Still allows you to fetch the underlying phobos types to rely on its strong pass / graph system

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