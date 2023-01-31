use std::{marker::PhantomData, any::{Any, TypeId}, sync::Arc};

use vulkan::{Recorder, vk};

use crate::{
    ColorLayout, DepthStencilLayout, GraphicsPipeline, Block, Member, BindingConfig, DepthConfig, StencilConfig, BlendConfig, VertexConfig, Shader, Primitive, UntypedBuffer, VertexBuffer, GpuPodRelaxed, BufferVariant,
};

// This is an active binding that is linked to a specific active pipeline
// We can use these bindings to set specific push constants or descriptor sets
pub struct ActiveBindings<'rp, 'r, 'gp> {
    recorder: &'rp Recorder<'r>,
    binding_config: &'gp BindingConfig,
    layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

impl<'rp, 'r, 'gp> ActiveBindings<'rp, 'r, 'gp> {
    // Create the active pipeline bindings from their raw components
    pub unsafe fn from_raw_parts(
        recorder: &'rp Recorder<'r>,
        graphics: &'gp GraphicsPipeline,
    ) -> Self {
        Self {
            recorder,
            binding_config: &graphics.binding_config(),
            layout: graphics.layout(),
            pipeline: graphics.raw(),
        }
    }
    
    // Update the whole push constant block
    pub fn set_push_constant<B: Block>(
        &mut self,
        block_name: &'static str,
        value: &B
    ) -> Option<()> {
        // We iterate because I want the user to be able to call "set_block" once for all recurrent block defs in each module
        for (kind, module_binding_config) in self.binding_config.iter() {
            if let Some((block, _type)) = &module_binding_config.push_constant {
                // Check if the block is the same as defined in the config
                // (which is also the same block as defined in the shader through reflection)
                if TypeId::of::<B>() == *_type {
                    // Set the block using cmdPushConstants
                    let internal = &value.serialize();
                    let pointer = internal as *const <B as Block>::Internal;


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
}

// This is an active graphics pipeline that we can use to render out objects to the parent render pass
// This is named a Rasterizer because it's what it was named in my old OpenGL implementation
pub struct ActiveRasterizer<'rp, 'r, 'gp> {
    recorder: &'rp Recorder<'r>,
    graphics: &'gp GraphicsPipeline,
}

impl<'rp, 'r, 'gp> ActiveRasterizer<'rp, 'r, 'gp> {
    // Create an active graphics pipeline from it's raw components
    pub(crate) unsafe fn from_raw_parts(
        recorder: &'rp Recorder<'r>,
        graphics: &'gp GraphicsPipeline,
    ) -> Self {
        Self { recorder, graphics }
    }

    // Get the graphics pipeline that is linked to this rasterizer
    pub fn graphics_pipeline(&self) -> &GraphicsPipeline {
        &self.graphics
    }

    // Get the depth config used when creating this rasterizer
    pub fn depth_config(&self) -> &DepthConfig {
        &self.graphics_pipeline().depth_config()
    }

    // Get the stencil config used when creating this rasterizer
    pub fn stencil_config(&self) -> &StencilConfig {
        &self.graphics_pipeline().stencil_config()
    }

    // Get the blend config used when creating this rasterizer
    pub fn blend_config(&self) -> &BlendConfig {
        &self.graphics_pipeline().blend_config()
    }

    // Get the vertex config used when creating this rasterizer
    pub fn vertex_config(&self) -> &VertexConfig {
        &self.graphics_pipeline().vertex_config()
    }

    // Get the internally used shader for this rasterizer
    pub fn shader(&self) -> &Shader {
        &self.graphics_pipeline().shader()
    }

    // Get the primitive config used when creating this rasterizer
    pub fn primitive(&self) -> &Primitive {
        &self.graphics_pipeline().primitive()
    }

    // Binds one vertex buffer to be used within the pipeline at a specific location
    pub fn bind_vertex_buffer<T: GpuPodRelaxed>(
        &mut self,
        buffer: &VertexBuffer<T>,
        binding: u32,
    ) {

    }

    // Bind multiple vertex buffers to be able to draw them in the draw command
    // This ignores buffers that are not defined as vertex buffers 
    // FIXME: IMPLEMENT RESOURCE TRACKING. THIS SHIT SUCKS ASS BRO 
    pub fn bind_vertex_buffers(
        &mut self,
        vertex_buffers: &[Option<UntypedBuffer>]
    ) {
        // FIXME: Make the size of this slice the lowest supported value of maxVertexInputAttributes
        let mut slice = [vk::Buffer::null(); 8];

        // Set the buffer handles inside the slice
        let mut next = 0;
        for buffer in vertex_buffers.iter() {
            if let Some(buffer) = buffer {
                if buffer.variant() == BufferVariant::Vertex {
                    slice[next] = buffer.raw().unwrap_or(vk::Buffer::null());
                    next += 1;
                }
            } else {
                next += 1;
            }
        }

        // Bind the vertex buffers to the rasterizer
        unsafe {
            self.recorder.cmd_bind_vertex_buffers(0, &slice, &[]);
        }
    }

    // Draw an array mesh using the currently bound vertex buffers without checking for safety
    pub unsafe fn draw_unchecked(
        &mut self,
        count: u32,
        bindings: &ActiveBindings
    ) {
        self.recorder.cmd_draw(count, 1, 0, 0);
    }

    // Draw an indexed mesh using the currently bound vertex buffers without checking for safety
    pub unsafe fn draw_indexed_unchecked(
        &mut self,
        count: u32,
        bindings: &ActiveBindings
    ) {
    }

    // Draw an array mesh using the currently bound vertex buffers
    pub fn draw(&mut self, count: u32, bindings: &ActiveBindings) {
        debug_assert_eq!(bindings.pipeline, self.graphics.raw());

        // Also check if we have vertex buffers bound

        // Only draw when we actually have vertices
        if count > 0 {
            unsafe {
                self.draw_unchecked(count, bindings);
            }
        }
    }

    // Draw an indexed mesh using the currently bound vertex buffers
    pub fn draw_indexed(&mut self, count: u32, bindings: &ActiveBindings) {
        debug_assert_eq!(bindings.pipeline, self.graphics.raw());
        // Also check if we have vertex buffers bound
        // Also check if we have index buffers bound
    }
}
