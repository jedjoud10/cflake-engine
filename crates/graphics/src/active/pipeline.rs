use ahash::AHashMap;
use wgpu::CommandEncoder;

use crate::{
    BindGroup, Buffer, BufferInfo, BufferMode, BufferUsage,
    ColorLayout, DepthStencilLayout, GpuPod, Graphics,
    GraphicsPipeline, ModuleKind, PushConstants, RenderCommand,
    TriangleBuffer, UniformBuffer, Vertex, VertexBuffer,
};
use std::{
    collections::hash_map::Entry,
    marker::PhantomData,
    ops::{Bound, Range, RangeBounds},
    sync::Arc,
};

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
    pub(crate) push_constant: &'a mut Vec<u8>,
    pub(crate) push_constant_global_offset: usize,
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

// Validate the bounds and convert them to byte bounds
fn convert<T: GpuPod, const TYPE: u32>(
    bounds: impl RangeBounds<usize>,
    buffer: &Buffer<T, TYPE>,
) -> Option<(Bound<u64>, Bound<u64>)> {
    let start = bounds.start_bound().cloned();
    let end = bounds.end_bound().cloned();
    buffer.convert_bounds_to_indices((start, end))?;
    let start = map(start, |x| x as u64 * buffer.stride() as u64);
    let end = map(end, |x| x as u64 * buffer.stride() as u64);
    Some((start, end))
}

impl<'a, 'r, 't, C: ColorLayout, DS: DepthStencilLayout>
    ActiveGraphicsPipeline<'a, 'r, 't, C, DS>
{
    // Assign a vertex buffer to a slot with a specific range
    pub fn set_vertex_buffer<V: Vertex>(
        &mut self,
        slot: u32,
        buffer: &'r VertexBuffer<V>,
        bounds: impl RangeBounds<usize>,
    ) -> Option<()> {
        // Validate the bounds and convert them to byte bounds
        let (start, end) = convert(bounds, buffer)?;

        // Store the command within the internal queue
        self.commands.push(RenderCommand::SetVertexBuffer {
            slot,
            buffer: buffer.as_untyped(),
            start,
            end,
        });

        Some(())
    }

    // Sets the active index buffer with a specific range
    pub fn set_index_buffer(
        &mut self,
        buffer: &'r TriangleBuffer<u32>,
        bounds: impl RangeBounds<usize>,
    ) -> Option<()> {
        // Validate the bounds and convert them to byte bounds
        let (start, end) = convert(bounds, buffer)?;

        // Store the command wtihin the internal queue
        self.commands.push(RenderCommand::SetIndexBuffer {
            buffer: buffer,
            start,
            end,
        });

        Some(())
    }

    // Set push constants before rendering
    pub fn set_push_constants(
        &mut self,
        callback: impl FnOnce(&mut PushConstants),
    ) {
        let shader = self.pipeline.shader();

        // Don't set the push constants if we don't have any to set
        let size = shader
            .reflected
            .push_constant_layouts
            .iter()
            .filter_map(|x| x.as_ref())
            .map(|x| x.size)
            .sum::<usize>();
        if size == 0 {
            return;
        }

        // Make sure we have enough bytes to store the push constants
        let pc = self.push_constant.len()
            - self.push_constant_global_offset;
        if pc < 1024 {
            self.push_constant
                .extend(std::iter::repeat(0).take(1024));
        }

        // Get the data that we will use
        let start = self.push_constant_global_offset;
        let end = size + start;
        let data = &mut self.push_constant[start..end];

        // Create push constants that we can set
        let mut push_constants = PushConstants {
            reflected: shader.reflected.clone(),
            ranges: Vec::new(),
            data,
        };

        // Let the user modify the push constant
        callback(&mut push_constants);

        // Iterate over all the push constant ranges and create commands
        let mut new_internal_offset = 0;
        for range in push_constants.ranges {
            self.commands.push(RenderCommand::SetPushConstants {
                stages: range.stages,
                size,
                global_offset: self.push_constant_global_offset,
                local_offset: range.offset,
            });
            new_internal_offset += size;
        }
        self.push_constant_global_offset += new_internal_offset;
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
        let cache = &self.graphics.0.cached;

        // Extract the resources from bind group (dissociate the lifetime)
        let BindGroup::<'_> {
            reflected,
            fill_ubos,
            mut resources,
            mut slots,
            mut ids,
            ..
        } = bind_group;

        // Check the cache, and create a new bind group
        let bind_group = match cache.bind_groups.entry(ids.clone()) {
            dashmap::mapref::entry::Entry::Occupied(occupied) => {
                occupied.get().clone()
            }
            dashmap::mapref::entry::Entry::Vacant(vacant) => {
                log::warn!("Did not find cached bind group (set = {binding}), creating new one...");

                // Get the bind group layout of the bind group
                let layout = &reflected.bind_group_layouts
                    [binding as usize]
                    .as_ref()
                    .unwrap();
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
