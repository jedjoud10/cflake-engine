use std::{marker::PhantomData, ops::Range};
use crate::{ColorLayout, DepthStencilLayout, UntypedBuffer, VertexBuffer, Vertex, TriangleBuffer, GraphicsPipeline, ActiveGraphicsPipeline};

// An active render pass is basically just a rasterizer that is used to bind
// multiple render pipelines so we can draw objects to the screen
pub struct ActiveRenderPass<
    'r,
    'c,
    'ds,
    C: ColorLayout,
    DS: DepthStencilLayout,
> {
    pub(crate) render_pass: wgpu::RenderPass<'r>,
    pub(crate) _phantom: PhantomData<&'c C>,
    pub(crate) _phantom2: PhantomData<&'ds DS>,
}

impl<'r, 'c, 'ds, C: ColorLayout, DS: DepthStencilLayout>
    ActiveRenderPass<'r, 'c, 'ds, C, DS>
{    
    // Bind a graphics pipeline, which takes mutable access of the render pass temporarily
    // Returns an active graphics pipeline that we can render to
    pub fn bind_pipeline<'a>(
        &'a mut self,
        pipeline: &'r GraphicsPipeline<C, DS>,
    ) -> ActiveGraphicsPipeline<'a, 'r, 'c, 'ds, C, DS>{
        self.render_pass.set_pipeline(pipeline.pipeline());

        ActiveGraphicsPipeline {
            render_pass: &mut self.render_pass,
            _phantom: PhantomData,
            _phantom2: PhantomData,
        }
    }
}
