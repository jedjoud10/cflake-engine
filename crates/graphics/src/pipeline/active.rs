use std::marker::PhantomData;

use vulkan::Recorder;

use crate::{Bindings, ColorLayout, DepthStencilLayout, GraphicsPipeline};

// This is an active graphics pipeline that we can use to render out objects to the parent render pass
// This is named a Rasterizer because it's what it was named in my old OpenGL implementation
pub struct ActiveGraphicsPipeline<'rp, 'r, 'gp> {
    recorder: &'rp mut Recorder<'r>,
    graphics: &'gp mut GraphicsPipeline,
}

impl<'rp, 'r, 'gp> ActiveGraphicsPipeline<'rp, 'r, 'gp>  {
    // Create an active graphics pipeline from it's raw components
    pub(crate) unsafe fn from_raw_parts(recorder: &'rp mut Recorder<'r>, graphics: &'gp mut GraphicsPipeline) -> Self {
        Self {
            recorder,
            graphics
        }
    }

    // Draw an array mesh using the currently bound vertex buffers without checking for safety
    pub unsafe fn draw_unchecked(&mut self, count: u32, bindings: &Bindings) {
        //self.recorder.cmd_push_constants(layout, stage_flags, offset, values);
        self.recorder.cmd_draw(count, 1, 0, 0);
    }

    // Draw an indexed mesh using the currently bound vertex buffers without checking for safety
    pub unsafe fn draw_indexed_unchecked(&mut self, count: u32, bindings: &Bindings) {
    }

    // Draw an array mesh using the currently bound vertex buffers
    pub fn draw(&mut self, count: u32, bindings: &Bindings) {
        if count > 0 {
            unsafe {
                self.draw_unchecked(count, bindings);
            }
        }
    }
    
    // Draw an indexed mesh using the currently bound vertex buffers
    pub fn draw_indexed(&mut self, count: u32, bindings: &Bindings) {
    }
}