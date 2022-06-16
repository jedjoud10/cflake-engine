use crate::{context::Context, prelude::Shader};
use super::{Canvas, Rasterizer};

// A painter will help us render specific shaded / colored objects onto the screen 
// A painter is like a rasterizer, but this time, it has a notion of what a shader is and what uniforms are
// Painters can be fetched from any canvas using the .paint() method to be able to draw onto that canvas
pub struct Painter<'canvas, 'shader, 'context> {
    pub(super) canvas: &'canvas mut Canvas,
    pub(super) context: &'context mut Context,
    pub(super) shader: &'shader mut Shader,
}

impl<'canvas, 'shader, 'context> Painter<'canvas, 'shader, 'context> {
    // Create a new rasterizer from this painter
    pub fn raster(&mut self) -> Rasterizer<'canvas, 'context> {

    }
}