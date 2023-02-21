use crate::{
    BindGroup, ColorLayout, DepthStencilLayout, Graphics,
    GraphicsPipeline, RenderCommand, TriangleBuffer, UntypedBuffer,
    Vertex, VertexBuffer, PushConstants, ModuleKind,
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
        self.commands.push(RenderCommand::SetVertexBuffer(
            slot,
            buffer.as_untyped(),
        ))
    }

    // Sets the active index buffer
    pub fn set_index_buffer(
        &mut self,
        buffer: &'r TriangleBuffer<u32>,
    ) {
        self.commands.push(RenderCommand::SetIndexBuffer(buffer))
    }

    // Set push constants before rendering
    pub fn set_push_constants(
        &mut self,
        callback: impl FnOnce(&mut PushConstants)
    ) {
        /*
        // Get shader and it's reflected data
        let shader = self.pipeline.shader();
        let reflected = &shader.reflected;
        
        // Check the shader's push constant layouts if they contain the field
        reflected
            .push_constant_layouts
            .iter()
            .filter_map(|x| x.as_ref())
            .map(|push_constant_layout| push_constant_layout.members.iter().any(|members| {

            }));
        */


        // Set push constants using the callback
        // Store the results of the push constants in memory
    }

    // Execute a callback that we will use to fill a bind group
    pub fn set_bind_group<'b>(
        &mut self,
        binding: u32,
        callback: impl FnOnce(&mut BindGroup<'b>),
    ) {
        let shader = self.pipeline.shader();

        // Check if the binding is valid
        let valid = shader
            .reflected
            .bind_group_layouts
            .get(binding as usize)
            .map(|x| x.is_some())
            .unwrap_or_default();

        // Don't set the bind group if it doesn't exist in the shader
        if !valid {
            return;
        }

        // Create a new bind group
        let mut bind_group = BindGroup {
            _phantom: PhantomData,
            reflected: shader.reflected.clone(),
            index: binding,
            resources: Vec::new(),
            ids: Vec::new(),
            slots: Vec::new(),
        };

        // Let the user modify the bind group 
        callback(&mut bind_group);


        let cache = &self.graphics.0.cached;
        let bind_group = match cache
            .bind_groups
            .entry(bind_group.ids.clone())
        {
            dashmap::mapref::entry::Entry::Occupied(occupied) => {
                occupied.get().clone()
            }
            dashmap::mapref::entry::Entry::Vacant(vacant) => {
                log::warn!("Did not find cached bind group (set = {binding}), creating new one...");

                let layout =
                    &shader.reflected.bind_group_layouts[binding as usize].as_ref().unwrap();
                let layout = self
                    .graphics
                    .0
                    .cached
                    .bind_group_layouts
                    .get(layout)
                    .unwrap();

                let entries = bind_group
                    .resources
                    .into_iter()
                    .zip(bind_group.slots.into_iter())
                    .map(|(resource, binding)| wgpu::BindGroupEntry {
                        binding,
                        resource,
                    })
                    .collect::<Vec<_>>();

                let desc = wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &layout,
                    entries: &entries,
                };

                let bind_group =
                    self.graphics.device().create_bind_group(&desc);
                let bind_group = Arc::new(bind_group);
                vacant.insert(bind_group.clone());
                bind_group
            }
        };
        self.commands
            .push(RenderCommand::SetBindGroup(binding, bind_group));
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
        self.commands
            .push(RenderCommand::DrawIndexed { indices, instances });
    }

    // Get the underlying graphics pipeline that is currently bound
    pub fn pipeline(&self) -> &GraphicsPipeline<C, DS> {
        self.pipeline
    }
}
