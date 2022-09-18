use crate::{
    context::Context,
    prelude::{Shader, Uniforms},
};

use super::{RasterSettings, Rasterizer};

// A viewport wrapper around raw OpenGL viewport
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub origin: vek::Vec2<u16>,
    pub extent: vek::Extent2<u16>,
}

// A display can be anything that has a specific size and that can be drawn into
// For now, displays are just the main game window and user created scoped canvases
pub trait Display: Sized {
    // Get the rendering viewport of this display
    fn viewport(&self) -> Viewport;

    // Get the size of the viewport
    fn size(&self) -> vek::Extent2<u16> {
        self.viewport().extent
    }

    // Get the framebuffer name of this display
    fn name(&self) -> u32;

    // Create a new rasterizer from this display
    fn rasterizer<'shader: 'uniforms, 'display, 'context, 'uniforms>(
        &'display mut self,
        ctx: &'context mut Context,
        shader: &'shader mut Shader,
        settings: RasterSettings,
    ) -> (Rasterizer<'display, 'context, Self>, Uniforms<'uniforms>) {
        // Set the viewport values
        unsafe {
            let v = self.viewport();
            gl::Viewport(
                v.origin.x as i32,
                v.origin.y as i32,
                v.extent.w as i32,
                v.extent.h as i32,
            );
        }

        // Create the new rasterizer and it's corresponding uniforms
        let uniforms = Uniforms::new(shader.as_mut());
        let rasterizer = unsafe { Rasterizer::new(self, ctx, settings) };
        (rasterizer, uniforms)
    }
}
