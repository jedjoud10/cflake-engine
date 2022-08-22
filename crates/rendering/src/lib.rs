pub extern crate opengl as gl;

pub mod buffer;
pub mod viewport;
pub mod context;
pub mod material;
pub mod mesh;
pub mod object;
pub mod others;
pub mod pipeline;
pub mod scene;
pub mod shader;
pub mod texture;
pub mod prelude {
    pub use super::buffer::*;
    pub use super::viewport::*;
    pub use super::context::*;
    pub use super::gl;
    pub use super::material::*;
    pub use super::mesh::*;
    pub use super::object::*;
    pub use super::others::*;
    pub use super::pipeline::*;
    pub use super::scene::*;
    pub use super::shader::*;
    pub use super::texture::*;
}
