use std::{marker::PhantomData, any::{Any, TypeId}};

use vulkan::Recorder;

use crate::{
    ColorLayout, DepthStencilLayout, GraphicsPipeline, Block, Member,
};

// This is an active graphics pipeline that we can use to render out objects to the parent render pass
// This is named a Rasterizer because it's what it was named in my old OpenGL implementation
pub struct ActiveGraphicsPipeline<'rp, 'r, 'gp> {
    recorder: &'rp mut Recorder<'r>,
    graphics: &'gp GraphicsPipeline,
}

impl<'rp, 'r, 'gp> ActiveGraphicsPipeline<'rp, 'r, 'gp> {
    // Create an active graphics pipeline from it's raw components
    pub(crate) unsafe fn from_raw_parts(
        recorder: &'rp mut Recorder<'r>,
        graphics: &'gp GraphicsPipeline,
    ) -> Self {
        Self { recorder, graphics }
    }

    // Update the whole push constant block
    pub fn set_block<B: Block>(
        &mut self,
        block_name: &'static str,
        value: &B
    ) -> Option<()> {
        // We iterate because I want the user to be able to call "set_block" once for all recurrent block defs in each module
        for (kind, module_binding_config) in &self.graphics.binding_config().0 {
            if let Some((block, _type)) = &module_binding_config.push_constant {
                // Check if the block is the same as defined in the config
                // (which is also the same block as defined in the shader through reflection)
                if TypeId::of::<B>() == *_type {
                    // Set the block using cmdPushConstants
                    let internal = value.serialize();
                    let boxed: Box<dyn Any> = Box::new(internal);
                
                    //self.push_constants.insert(*kind, (Cell::new(true), boxed));
                } else {
                    // Block definition mismatch
                    return None;
                }
            } else {
                // Block not defined
                return None;
            }
        }

        Some(())
    }

    // Update a sub-range of push constants within a push constant block
    // This assumes that the variable is set as dynamic within the defintion of Block "B"
    pub fn set<M: Member, B: Block>(
        &mut self,
        block_name: &'static str,
        var_name: &'static str,
        value: &M
    ) -> Option<()> {
        todo!()
    }

    // Draw an array mesh using the currently bound vertex buffers without checking for safety
    pub unsafe fn draw_unchecked(
        &mut self,
        count: u32,
    ) {
        self.recorder.cmd_draw(count, 1, 0, 0);
    }

    // Draw an indexed mesh using the currently bound vertex buffers without checking for safety
    pub unsafe fn draw_indexed_unchecked(
        &mut self,
        count: u32,
    ) {
    }

    // Draw an array mesh using the currently bound vertex buffers
    pub fn draw(&mut self, count: u32) {
        if count > 0 {
            unsafe {
                self.draw_unchecked(count);
            }
        }
    }

    // Draw an indexed mesh using the currently bound vertex buffers
    pub fn draw_indexed(&mut self, count: u32) {}
}
