use crate::{
    ActiveRasterizer, ColorLayout,
    DepthStencilLayout, GraphicsPipeline, Viewport, ActiveBindings,
};
use std::marker::PhantomData;
use crate::vulkan::{vk, Recorder};

// An active render pass is basically just a rasterize that is used to bind
// multiple render graphical pipelines so we can draw objects to the screen
pub struct ActiveRenderPass<
    'r,
    'c,
    'ds,
    C: ColorLayout,
    DS: DepthStencilLayout,
> {
    viewport: Viewport,
    recorder: Recorder<'r>,
    _phantom_color_layout: PhantomData<&'c C>,
    _phantom_depth_stencil_layout: PhantomData<&'ds DS>,
}

impl<'r, 'c, 'ds, C: ColorLayout, DS: DepthStencilLayout>
    ActiveRenderPass<'r, 'c, 'ds, C, DS>
{
    // Create an active render pass from it's raw components
    pub(crate) unsafe fn from_raw_parts(
        viewport: Viewport,
        recorder: Recorder<'r>,
    ) -> Self {
        Self {
            viewport,
            recorder,
            _phantom_color_layout: PhantomData,
            _phantom_depth_stencil_layout: PhantomData,
        }
    }

    // Bind a graphics pipeline, which takes mutable access of the rasterizer temporarily
    // I made it return an ActiveGraphicsPipeline so we can bind multiple pipelines in the same render pass
    pub fn bind_pipeline<'gp: 'rp, 'rp>(
        &'rp mut self,
        pipeline: &'gp GraphicsPipeline,
    ) -> (ActiveRasterizer<'rp, 'r, 'gp>, ActiveBindings<'rp, 'r, 'gp>) {
        // Set dynamic state (viewport and scissor only)
        unsafe fn set_dynamic_state(
            recorder: &mut Recorder,
            viewport: &Viewport,
        ) {
            recorder.cmd_set_viewport(
                viewport.origin.x as f32,
                viewport.origin.y as f32,
                viewport.extent.w as f32,
                viewport.extent.h as f32,
                0.01,
                1.0,
            );
            recorder.cmd_set_scissor(
                0,
                0,
                viewport.extent.w,
                viewport.extent.h,
            );
        }

        unsafe {
            // Bind the Vulkan pipeline and update state
            self.recorder.cmd_bind_pipeline(
                pipeline.raw(),
                vk::PipelineBindPoint::GRAPHICS,
            );
            set_dynamic_state(&mut self.recorder, &self.viewport);

            // Create the actige graphics pipeline struct
            
        (ActiveRasterizer::from_raw_parts(
                    &self.recorder,
                    pipeline,
                ), ActiveBindings::from_raw_parts(&self.recorder, pipeline))
        }
    }

    // Stop the render pass, and return the recorder that must be sent to the GPU
    // TODO: Automatically do this on drop or nahski?
    pub fn end(mut self) -> Recorder<'r> {
        unsafe {
            self.recorder.cmd_end_render_pass();
            self.recorder
        }
    }
}
