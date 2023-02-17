use std::{marker::PhantomData, ops::Range};
use crate::{ColorLayout, DepthStencilLayout, UntypedBuffer, VertexBuffer, Vertex, TriangleBuffer, GraphicsPipeline};

// An active graphics pipeline that is bound to a render pass that we can use to render
pub struct ActiveGraphicsPipeline<
    'a, 
    'r,
    'c,
    'ds,
    C: ColorLayout,
    DS: DepthStencilLayout,
> {
    pub(crate) pipeline: &'r GraphicsPipeline<C, DS>,
    pub(crate) render_pass: &'a mut wgpu::RenderPass<'r>,
    pub(crate) _phantom: PhantomData<&'c C>,
    pub(crate) _phantom2: PhantomData<&'ds DS>,
}

impl<'a, 'r, 'c, 'ds, C: ColorLayout, DS: DepthStencilLayout>
    ActiveGraphicsPipeline<'a, 'r, 'c, 'ds, C, DS>
{    
    // Assign a vertex buffer to a slot
    pub fn set_vertex_buffer<V: Vertex>(&mut self, slot: u32, buffer: &'r VertexBuffer<V>) {
        self.render_pass.set_vertex_buffer(slot, buffer.raw().slice(..));
    }
    
    // Sets the active index buffer
    pub fn set_index_buffer(&mut self, buffer: &'r TriangleBuffer<u32>) {
        self.render_pass.set_index_buffer(buffer.raw().slice(..), wgpu::IndexFormat::Uint32);
    }

    // Draw a number of primitives using the currently bound vertex buffers
    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.render_pass.draw(vertices, instances);
    }

    // Draw a number of primitives using the currently bound vertex buffers and index buffer
    pub fn draw_indexed(&mut self, indices: Range<u32>, instances: Range<u32>) {
        self.render_pass.draw_indexed(indices, 0, instances);
    }

    // Get the underlying graphics pipeline that is currently bound
    pub fn pipeline(&self) -> &GraphicsPipeline<C, DS> {
        self.pipeline
    }
}