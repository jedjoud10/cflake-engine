use std::marker::PhantomData;

use vulkan::Recorder;

use crate::{Uniforms, ColorLayout, DepthStencilLayout};

// This is an active graphics pipeline that we can use to render out objects to the parent render pass
// This is named a Rasterizer because it's what it was named in my old OpenGL implementation
pub struct ActiveGraphicsPipeline<'rp, 'r, 'c, 'ds, C: ColorLayout, DS: DepthStencilLayout> {
    recorder: &'rp mut Recorder<'r>,
    _phantom_color_layout: PhantomData<&'c C>,
    _phantom_depth_stencil_layout: PhantomData<&'ds DS>,
}

impl<'rp, 'r, 'c, 'ds, C: ColorLayout, DS: DepthStencilLayout> ActiveGraphicsPipeline<'rp, 'r, 'c, 'ds, C, DS>  {
    // Create an active graphics pipeline from it's raw components
    pub(crate) unsafe fn from_raw_parts(recorder: &'rp mut Recorder<'r>) -> Self {
        Self {
            recorder,
            _phantom_color_layout: PhantomData,
            _phantom_depth_stencil_layout: PhantomData,
        }
    }

    // Draw an array mesh using the currently bound vertex buffers
    pub fn draw(&mut self, count: u32, unfiroms: &Uniforms) {

    }
    
    // Draw an indexed mesh using the currently bound vertex buffers
    pub fn draw_indexed(&mut self, count: u32, uniforms: &Uniforms) {
    }
}