use crate::{
    BindGroup, ColorLayout, DepthStencilLayout, GraphicsPipeline,
    TriangleBuffer, UntypedBuffer, Vertex, VertexBuffer, Graphics, RenderCommand,
};
use std::{marker::PhantomData, ops::Range, sync::Arc};

// An active graphics pipeline that is bound to a render pass that we can use to render
pub struct ActiveGraphicsPipeline<
    'a,
    'r,
    't,
    C: ColorLayout,
    DS: DepthStencilLayout,
> {
    pub(crate) pipeline: &'r GraphicsPipeline<C, DS>,
    pub(crate) commands: &'a mut Vec<RenderCommand<'r, C, DS>>,
    pub(crate) graphics: &'r Graphics,
    pub(crate) _phantom: PhantomData<&'t C>,
    pub(crate) _phantom2: PhantomData<&'t DS>,
}

impl<'a, 'r, 't, C: ColorLayout, DS: DepthStencilLayout>
    ActiveGraphicsPipeline<'a, 'r, 't, C, DS>
{
    // Assign a vertex buffer to a slot
    pub fn set_vertex_buffer<V: Vertex>(
        &mut self,
        slot: u32,
        buffer: &'r VertexBuffer<V>,
    ) {
        self.commands.push(RenderCommand::SetVertexBuffer(slot, buffer.as_untyped()))
    }

    // Sets the active index buffer
    pub fn set_index_buffer(
        &mut self,
        buffer: &'r TriangleBuffer<u32>,
    ) {
        self.commands.push(RenderCommand::SetIndexBuffer(buffer))
    }

    // Execute a callback that we will use to fill a bind group
    pub fn set_bind_group(
        &mut self,
        binding: u32,
        callback: impl FnOnce(&mut BindGroup<'a>),
    ) {
        let shader = self.pipeline.shader();
        if (binding as usize) >= shader.reflected.groups.len() {
            return;
        }

        let mut bind_group = BindGroup {
            _phantom: PhantomData,
            shader: shader,
            index: binding,
            resources: Vec::new(),
            ids: Vec::new(),
        };

        callback(&mut bind_group);

        let cache = &self.graphics.0.cached;
        let bind_group = match cache.bind_groups.entry(bind_group.ids.clone()) {
            dashmap::mapref::entry::Entry::Occupied(occupied) => {
                occupied.get().clone()
            },
            dashmap::mapref::entry::Entry::Vacant(vacant) => {
                log::warn!("Did not find cached bind group (set = {binding}), creating new one...");

                let layout = &shader.reflected.groups[binding as usize];
                let layout = self.graphics.0.cached.bind_group_layouts.get(layout).unwrap();

                let entries = bind_group.resources.into_iter().map(|x| {
                    wgpu::BindGroupEntry {
                        binding,
                        resource: x,
                    }
                }).collect::<Vec<_>>();

                let desc = wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &layout,
                    entries: &entries,
                };

                let bind_group = self.graphics
                    .device().create_bind_group(&desc);
                let bind_group = Arc::new(bind_group);
                vacant.insert(bind_group.clone());
                bind_group
            },
        };
        self.commands.push(RenderCommand::SetBindGroup(binding, bind_group));

        /*

        */

        // Hash the entries from the bind group
        // Check if we have a bind group with the same entries
        //      Create a new one if not
        
        // Set the bind group
        //self.render_pass.set_bind_group(binding, &bind_group2, &[])
    }

    // Draw a number of primitives using the currently bound vertex buffers
    pub fn draw(
        &mut self,
        vertices: Range<u32>,
        instances: Range<u32>,
    ) {
        self.commands.push(RenderCommand::Draw {
            vertices,
            instances,
        });
    }

    // Draw a number of primitives using the currently bound vertex buffers and index buffer
    pub fn draw_indexed(
        &mut self,
        indices: Range<u32>,
        instances: Range<u32>,
    ) {
        self.commands.push(RenderCommand::DrawIndexed {
            indices,
            instances
        });
    }

    // Get the underlying graphics pipeline that is currently bound
    pub fn pipeline(&self) -> &GraphicsPipeline<C, DS> {
        self.pipeline
    }
}
