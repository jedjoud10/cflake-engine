// Re-export the OpenGL crate
pub extern crate opengl as gl;

mod others;
pub use others::*;
pub mod canvas;
pub mod context;
pub mod material;
pub mod mesh;
pub mod object;
pub mod scene;
pub mod shader;
pub mod texture;

pub mod prelude {
    pub use super::buffer::*;
    pub use super::commons::*;
    pub use super::canvas::*;
    pub use super::context::*;
    pub use super::material::*;
    pub use super::mesh::*;
    pub use super::object::*;
    pub use super::scene::*;
    pub use super::shader::*;
    pub use super::texture::*;
}