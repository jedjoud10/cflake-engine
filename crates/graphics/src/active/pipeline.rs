use ahash::AHashMap;

use crate::{
    BindGroup, ColorLayout, DepthStencilLayout, Graphics,
    GraphicsPipeline, RenderCommand, TriangleBuffer, UntypedBuffer,
    Vertex, VertexBuffer, PushConstants, ModuleKind, UniformBuffer, BufferMode, BufferUsage, GpuPod,
};
use std::{marker::PhantomData, ops::{Range, RangeBounds, Bound}, sync::Arc, collections::hash_map::Entry};

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

// Map bound value since Rust doesn't have that stabilized yet
fn map<T, U, F: FnOnce(T) -> U>(bound: Bound<T>, map: F) -> Bound<U> {
    match bound {
        Bound::Included(x) => Bound::Included(map(x)),
        Bound::Excluded(x) => Bound::Excluded(map(x)),
        Bound::Unbounded => Bound::Unbounded,
    }
}

// Convert some buffer range bounds to byte starts / ends
fn convert(bounds: impl RangeBounds<usize>, stride: usize) -> (Bound<u64>, Bound<u64>) {
    let stride = stride as u64;
    let start = map(bounds.start_bound().cloned(), |x| x as u64 * stride);
    let end = map(bounds.end_bound().cloned(), |x| x as u64 * stride);
    (start, end)
}

impl<'a, 'r, 't, C: ColorLayout, DS: DepthStencilLayout>
    ActiveGraphicsPipeline<'a, 'r, 't, C, DS>
{
    // Assign a vertex buffer to a slot with a specific range
    // TODO: CHECK IF RANGE BOUNDS IS VALID
    pub fn set_vertex_buffer<V: Vertex>(
        &mut self,
        slot: u32,
        buffer: &'r VertexBuffer<V>,
        bounds: impl RangeBounds<usize>,
    ) {
        let (start, end) = convert(bounds, buffer.stride());
        self.commands.push(RenderCommand::SetVertexBuffer {
            slot,
            buffer: buffer.as_untyped(),
            start,
            end
        })
    }

    // Sets the active index buffer with a specific range
    // TODO: CHECK IF RANGE BOUNDS IS VALID
    pub fn set_index_buffer(
        &mut self,
        buffer: &'r TriangleBuffer<u32>,
        bounds: impl RangeBounds<usize>,
    ) {
        let (start, end) = convert(bounds, buffer.stride());
        self.commands.push(RenderCommand::SetIndexBuffer {
            buffer: buffer,
            start,
            end
        })
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

        // ON DIT NON A L'INTIMIDATION
        if binding >= 4 {
            return;
        }

        // Get the bind group layout from the shader
        let bind_group_layout = shader
            .reflected
            .bind_group_layouts
            .get(binding as usize)
            .unwrap();

        // Don't set the bind group if it doesn't exist in the shader
        let Some(bind_group_layout) = bind_group_layout else {
            return;
        };

        // Get the number of resources that we will bind so we can pre-allocate the vectors
        let count = bind_group_layout.bind_entry_layouts.len();

        // Create a new bind group
        let mut bind_group = BindGroup {
            _phantom: PhantomData,
            reflected: shader.reflected.clone(),
            index: binding,
            resources: Vec::with_capacity(count),
            ids: Vec::with_capacity(count),
            slots: Vec::with_capacity(count),
            fill_ubos: Vec::with_capacity(count),
        };

        // Let the user modify the bind group 
        callback(&mut bind_group);

        // Check the cache, and create a new fill UBOs if needed
        // This will also fill the buffers, but it won't bind them
        let cache = &self.graphics.0.cached;
        let mut cached_ubos = cache.uniform_buffers.lock();

        // Extract the resources from bind group (dissociate the lifetime)
        let BindGroup::<'_> {
            reflected,
            fill_ubos,
            mut resources,
            mut slots,
            mut ids,
            ..
        } = bind_group;

        // Contains the indices of a free UBO buffer of a specific layout that we can use
        let mut filled_up_ubos = Vec::<usize>::with_capacity(fill_ubos.len());

        for (data, layout) in fill_ubos.iter() {
            match cached_ubos.entry((binding, layout.clone())) {
                // There is an already existing UBO buffer with the same layout and bind group, fill it up
                Entry::Occupied(mut occupied) => {
                    // Check if there's an unused buffer that we can use 
                    let buffers = occupied.get_mut();
                    let buffer = buffers.iter_mut().enumerate().find(|(_, (_, x))| *x);

                    if let Some((index, (buffer, free))) = buffer {
                        buffer.write(&data, 0).unwrap();
                        *free = false;
                        filled_up_ubos.push(index);
                    } else {
                        // Add a new unused buffer
                        log::warn!("Did not find free fill buffer for bind group (set = {binding}), allocating a new one...");
                        let buffer = UniformBuffer::<u8>::from_slice(
                            &self.graphics,
                            &data,
                            BufferMode::Dynamic,
                            BufferUsage::WRITE
                        ).unwrap();
                        filled_up_ubos.push(buffers.len());
                        buffers.push((buffer, true));
                    }                    
                },

                // Create a new UBO with the specified layout and bind group
                Entry::Vacant(vacant) => {
                    log::warn!("Did not find fill buffers ring buffer for bind group (set = {binding}), allocating a new one...");
                    let buffer = UniformBuffer::<u8>::from_slice(
                        &self.graphics,
                        &data,
                        BufferMode::Dynamic,
                        BufferUsage::WRITE
                    ).unwrap();
                    vacant.insert(vec![(buffer, true)]);
                    filled_up_ubos.push(0);
                },
            }
        }

        // Update the bind group IDs based on the fetched UBOS
        // (very important for the next step)
        for (index, (_, layout)) in fill_ubos.iter().enumerate() {
            let buffers = cached_ubos.get(&(binding, layout.clone())).unwrap();
            let (buffer, _) = &buffers[filled_up_ubos[index]];

            // Get values needed for the bind entry
            let id = buffer.raw().global_id();
            let buffer_binding = buffer.raw().as_entire_buffer_binding();
            let resource = wgpu::BindingResource::Buffer(buffer_binding);

            // Save the bind entry for later
            resources.push(resource);
            ids.push(id);
            slots.push(layout.binding);
        } 

        // Check the cache, and create a new bind group
        let bind_group = match cache
            .bind_groups
            .entry(ids.clone())
        {
            dashmap::mapref::entry::Entry::Occupied(occupied) => {
                occupied.get().clone()
            }
            dashmap::mapref::entry::Entry::Vacant(vacant) => {
                log::warn!("Did not find cached bind group (set = {binding}), creating new one...");

                // Get the bind group layout of the bind group
                let layout =
                    &reflected.bind_group_layouts[binding as usize].as_ref().unwrap();
                let layout = self
                    .graphics
                    .0
                    .cached
                    .bind_group_layouts
                    .get(layout)
                    .unwrap();

                // Get the bind group entries
                let entries = resources
                    .into_iter()
                    .zip(slots.into_iter())
                    .map(|(resource, binding)| wgpu::BindGroupEntry {
                        binding,
                        resource,
                    })
                    .collect::<Vec<_>>();

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
