#![feature(concat_idents)]
pub extern crate opengl as gl;

pub mod buffer;
pub mod context;
pub mod display;
pub mod material;
pub mod mesh;
pub mod others;
pub mod painter;
pub mod pipeline;
pub mod query;
pub mod scene;
pub mod shader;
pub mod texture;
pub mod prelude {
    pub use super::buffer::*;
    pub use super::context::*;
    pub use super::display::*;
    pub use super::gl;
    pub use super::material::*;
    pub use super::mesh::*;
    pub use super::others::*;
    pub use super::painter::*;
    pub use super::pipeline::*;
    pub use super::query::*;
    pub use super::scene::*;
    pub use super::shader::*;
    pub use super::texture::*;
}
