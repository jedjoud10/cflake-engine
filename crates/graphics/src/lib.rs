#![warn(missing_docs)]

//! TODO: Docs

/// Contains everything about active (bound) render/compute pipelines
pub mod active;

/// Contains everything about buffers, how to read/write from/them, and the main [Buffer<T>] wrapper
pub mod buffer;

/// Context that contains the main [Window] and [Graphics] contexts
pub mod context;

/// Convenience constant time formatting and types
pub mod format;

/// Main compute/render passes and their settings
pub mod pass;

/// Main compute/render pipelines and their settings
pub mod pipeline;

/// Plain-old-data convenience types and rewrites of WGPU types to implement POD
pub mod pod;

/// Shader compilation and layouting
pub mod shader;


/// Main plugin that will create the wgpu context
pub mod plugin;

/// Anything related to textures (other than formats)
pub mod texture;

/// Tests :3
pub mod tests;

/// Re-exports everything
pub mod prelude {
    pub use crate::active::*;
    pub use crate::buffer::*;
    pub use crate::context::*;
    pub use crate::format::*;
    pub use crate::pass::*;
    pub use crate::pipeline::*;
    pub use crate::pod::*;
    pub use crate::shader::*;
    pub use crate::plugin::*;
    pub use crate::texture::*;
}

pub use prelude::*;