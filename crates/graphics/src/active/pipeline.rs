use ahash::AHashMap;

use crate::{
    BindGroup, ColorLayout, DepthStencilLayout, Graphics,
    GraphicsPipeline, RenderCommand, TriangleBuffer, UntypedBuffer,
    Vertex, VertexBuffer, PushConstants, ModuleKind, UniformBuffer, BufferMode, BufferUsage,
};
use std::{marker::PhantomData, ops::Range, sync::Arc, collections::hash_map::Entry};

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
        let shader = self.pipeline.shader();

        // Don't set the push constants if we don't have any to set
        let valid = shader.reflected.push_constant_layouts.iter().any(|x| x.is_some());
        if !valid {
            return;
        }

        // Create push constants that we can set 
        let mut push_constants = PushConstants {
            reflected: shader.reflected.clone(),
            offsets: Vec::new(),
            data: Vec::new(),
            stages: Vec::new(),
            _phantom: PhantomData,
        };

        // Let the user modify the push constant
        callback(&mut push_constants);

        // Fetch data back from push constants
        let offsets = push_constants.offsets;
        let data = push_constants.data;
        let stages = push_constants.stages;

        // Create the render commands for settings for push constants
        let iter = stages.into_iter().zip(offsets.into_iter().zip(data.into_iter()));
        for (stages, (offset, data)) in iter {
            self.commands.push(RenderCommand::SetPushConstants {
                stages,
                offset,
                data
            })
        }
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
            fill_ubos: AHashMap::new(),
        };

        // Let the user modify the bind group 
        callback(&mut bind_group);

        // Check the cache, and create a new fill UBOs if needed
        // This will also fill the buffers, but it won't bind them
        let cache = &self.graphics.0.cached;
        let mut ubos = cache.fill_buffers_ubo.lock();
        let fill_ubos = &bind_group.fill_ubos;
        for (name, (data, layout)) in fill_ubos.iter() {
            match ubos.entry((binding, layout.clone())) {
                // There is an already existing UBO buffer with the same layout and bind group, fill it up
                Entry::Occupied(mut occupied) => {
                    let buffer = occupied.get_mut();
                    buffer.write(&data, 0).unwrap();
                },

                // Create a new UBO with the specified layout and bind group
                Entry::Vacant(vacant) => {
                    let buffer = UniformBuffer::<u8>::from_slice(
                        &self.graphics,
                        &data,
                        BufferMode::Resizable,
                        BufferUsage::Write
                    ).unwrap();
                },
            }
        }

        // Check the cache, and create a new bind group
        let bind_group = match cache
            .bind_groups
            .entry(bind_group.ids.clone())
        {
            dashmap::mapref::entry::Entry::Occupied(occupied) => {
                occupied.get().clone()
            }
            dashmap::mapref::entry::Entry::Vacant(vacant) => {
                log::warn!("Did not find cached bind group (set = {binding}), creating new one...");

                // Get the bind group layout of the bind group
                let layout =
                    &shader.reflected.bind_group_layouts[binding as usize].as_ref().unwrap();
                let layout = self
                    .graphics
                    .0
                    .cached
                    .bind_group_layouts
                    .get(layout)
                    .unwrap();

                // Get the bind group entries
                let mut entries = bind_group
                    .resources
                    .into_iter()
                    .zip(bind_group.slots.into_iter())
                    .map(|(resource, binding)| wgpu::BindGroupEntry {
                        binding,
                        resource,
                    })
                    .collect::<Vec<_>>();

                // Take in consideration the fill buffer UBOs
                for (name, (data, layout)) in fill_ubos.iter() {
                    // Get the buffer back from cache
                    let buffer = ubos.get(&(binding, layout.clone())).unwrap();
                    
                    entries.push(wgpu::BindGroupEntry {
                        binding,
                        resource: wgpu::BindingResource::Buffer(buffer.raw().as_entire_buffer_binding()),
                    });
                } 

                // Create a bind group descriptor of the entries 
                let desc = wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &layout,
                    entries: &entries,
                };

                // Create the bind group and cache it for later use
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
