use super::{Canvas, RasterSettings, Rasterizer};
use crate::{
    context::Context,
    prelude::{Program, Shader, Uniforms},
};

// A painter will help us render specific shaded / colored objects onto the screen
// A painter is like a rasterizer, but this time, it has a notion of what a shader is and what uniforms are
// Painters can be fetched from any canvas using the .paint() method to be able to draw onto that canvas
pub struct Painter<'canvas, 'shader, 'context> {
    // A painter consists of a geometry rasterizer
    pub(super) rasterizer: Rasterizer<'canvas, 'context>,

    // And some shader uniforms (to color/shader the geoemtry)
    pub(super) uniforms: Uniforms<'shader>,
}

impl<'canvas, 'shader, 'context> Painter<'canvas, 'shader, 'context> {
    // Create a new pass that will set the uniforms of the underlying shader
    // After we set the uniforms, we can use it to render the objects using the rasterizer
    pub fn pass(
        &mut self,
        populate: impl FnOnce(&mut Uniforms<'shader>),
        rasterize: impl FnOnce(&mut Rasterizer<'canvas, 'context>),
    ) {
        populate(&mut self.uniforms);
        rasterize(&mut self.rasterizer);        
    }
}
