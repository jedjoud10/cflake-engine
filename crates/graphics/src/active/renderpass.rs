use crate::{
    ActiveGraphicsPipeline, ColorLayout, DepthStencilLayout,
    GraphicsPipeline, TriangleBuffer, UntypedBuffer, Vertex,
    VertexBuffer, Graphics, RenderCommand,
};
use std::{marker::PhantomData, ops::Range, sync::Arc};

// An active render pass is basically just a rasterizer that is used to bind
// multiple render pipelines so we can draw objects to the screen
pub struct ActiveRenderPass<
    'r,
    't,
    C: ColorLayout,
    DS: DepthStencilLayout,
> {
    pub(crate) render_pass: Option<wgpu::RenderPass<'r>>,
    pub(crate) commands: Vec<RenderCommand<'r, C, DS>>,
    pub(crate) graphics: &'r Graphics,
    pub(crate) _phantom: PhantomData<&'t C>,
    pub(crate) _phantom2: PhantomData<&'t DS>,
}

impl<'r, 't, C: ColorLayout, DS: DepthStencilLayout>
    ActiveRenderPass<'r, 't, C, DS>
{
    // Bind a graphics pipeline, which takes mutable access of the render pass temporarily
    // Returns an active graphics pipeline that we can render to
    pub fn bind_pipeline<'a>(
        &'a mut self,
        pipeline: &'r GraphicsPipeline<C, DS>,
    ) -> ActiveGraphicsPipeline<'a, 'r, 't, C, DS> {
        self.commands.push(RenderCommand::BindPipeline(&pipeline));
        ActiveGraphicsPipeline {
            _phantom: PhantomData,
            _phantom2: PhantomData,
            pipeline: &pipeline,
            graphics: self.graphics,
            commands: &mut self.commands,
        }
    }
}

impl<'r, 't, C: ColorLayout, DS: DepthStencilLayout> Drop for ActiveRenderPass<'r, 't, C, DS> {
    fn drop(&mut self) {
        let taken = self.render_pass.take().unwrap();
        super::record(
            taken,
            &self.commands
        )
    }
}