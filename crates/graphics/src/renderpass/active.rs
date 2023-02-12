use std::{marker::PhantomData, ops::Range};

use crate::{ColorLayout, DepthStencilLayout, UntypedBuffer, VertexBuffer, Vertex, TriangleBuffer, GraphicsPipeline};

// An active render pass is basically just a rasterizer that is used to bind
// multiple render pipelines so we can draw objects to the screen
pub struct ActiveRenderPass<
    'r,
    'c,
    'ds,
    C: ColorLayout,
    DS: DepthStencilLayout,
> {
    render_pass: wgpu::RenderPass<'r>,
    _phantom: PhantomData<&'c C>,
    _phantom2: PhantomData<&'ds DS>,
}

impl<'r, 'c, 'ds, C: ColorLayout, DS: DepthStencilLayout>
    ActiveRenderPass<'r, 'c, 'ds, C, DS>
{
    // Assign a vertex buffer to a slot
    pub fn set_vertex_buffer<V: Vertex>(&mut self, slot: u32, buffer: &'r VertexBuffer<V::Storage>) {
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

    // Bind a graphics pipeline, which takes mutable access of the rasterizer temporarily
    pub fn bind_pipeline<'gp: 'rp, 'rp>(
        &'rp mut self,
        pipeline: &'gp GraphicsPipeline<C, DS>,
    ) {
    }
}
