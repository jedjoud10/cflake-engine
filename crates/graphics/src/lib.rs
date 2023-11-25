#![warn(missing_docs)]

//! TODO: Docs

pub mod active;
pub mod buffer;
pub mod context;
pub mod format;
pub mod pass;
pub mod pipeline;
pub mod pod;
pub mod shader;
pub mod plugin;
pub mod tests;
pub mod texture;

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