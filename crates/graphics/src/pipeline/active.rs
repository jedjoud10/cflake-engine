use crate::{
    BindGroup, ColorLayout, DepthStencilLayout, GraphicsPipeline,
    TriangleBuffer, UntypedBuffer, Vertex, VertexBuffer, Graphics,
};
use std::{marker::PhantomData, ops::Range};

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
    pub(crate) graphics: &'r Graphics,
    pub(crate) _phantom: PhantomData<&'c C>,
    pub(crate) _phantom2: PhantomData<&'ds DS>,
}

impl<'a, 'r, 'c, 'ds, C: ColorLayout, DS: DepthStencilLayout>
    ActiveGraphicsPipeline<'a, 'r, 'c, 'ds, C, DS>
{
    // Assign a vertex buffer to a slot
    pub fn set_vertex_buffer<V: Vertex>(
        &mut self,
        slot: u32,
        buffer: &'r VertexBuffer<V>,
    ) {
        self.render_pass
            .set_vertex_buffer(slot, buffer.raw().slice(..));
    }

    // Sets the active index buffer
    pub fn set_index_buffer(
        &mut self,
        buffer: &'r TriangleBuffer<u32>,
    ) {
        self.render_pass.set_index_buffer(
            buffer.raw().slice(..),
            wgpu::IndexFormat::Uint32,
        );
    }

    // Execute a callback that we will use to fill a bind group
    pub fn set_bind_group(
        &mut self,
        binding: u32,
        callback: impl FnOnce(&mut BindGroup<'a>),
    ) {
        let shader = self.pipeline.shader();
        let mut bind_group = BindGroup {
            _phantom: PhantomData,
            shader: shader,
            index: binding,
            resources: Vec::new(),
            ids: Vec::new(),
        };

        callback(&mut bind_group);

        let cache = &self.graphics.0.cached;

        /*
        let bind_group = match cache.bind_groups.entry(bind_group.ids.clone()) {
            dashmap::mapref::entry::Entry::Occupied(occupied) => {
                occupied.get().clone()
            },
            dashmap::mapref::entry::Entry::Vacant(vacant) => {
                log::warn!("Did not find cached bind group, creating new one...");

                let bind_group = self.graphics.device();

                todo!()
            },
        };
        */

        // Hash the entries from the bind group
        // Check if we have a bind group with the same entries
        //      Create a new one if not
        
        // Set the bind group
        self.render_pass.set_bind_group(binding, &bind_group, &[])
    }

    // Draw a number of primitives using the currently bound vertex buffers
    pub fn draw(
        &mut self,
        vertices: Range<u32>,
        instances: Range<u32>,
    ) {
        self.render_pass.draw(vertices, instances);
    }

    // Draw a number of primitives using the currently bound vertex buffers and index buffer
    pub fn draw_indexed(
        &mut self,
        indices: Range<u32>,
        instances: Range<u32>,
    ) {
        self.render_pass.draw_indexed(indices, 0, instances);
    }

    // Get the underlying graphics pipeline that is currently bound
    pub fn pipeline(&self) -> &GraphicsPipeline<C, DS> {
        self.pipeline
    }
}
