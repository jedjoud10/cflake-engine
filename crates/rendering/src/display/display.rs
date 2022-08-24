use crate::{
    context::Context,
    prelude::{Shader, Uniforms},
};

use super::{RasterSettings, Rasterizer, Viewport};

// A display can be anything that has a specific size and that can be drawn into
// For now, displays are just the main game window and user created scoped canvases
pub trait Display: Sized {
    fn viewport(&self) -> Viewport;
    fn size(&self) -> vek::Extent2<u16> {
        self.viewport().extent
    }
    fn name(&self) -> u32;
    fn rasterizer<'shader: 'uniforms, 'display, 'context, 'uniforms>(
        &'display mut self,
        ctx: &'context mut Context,
        shader: &'shader mut Shader,
        settings: RasterSettings,
    ) -> (Rasterizer<'display, 'context, Self>, Uniforms<'uniforms>) {
        // Create the new rasterizer and it's corresponding uniforms
        let uniforms = Uniforms::new(shader.as_mut(), ctx);
        let rasterizer = unsafe { Rasterizer::new(self, ctx, settings) };
        (rasterizer, uniforms)
    }
}
