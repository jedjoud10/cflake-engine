use ahash::AHashMap;
use utils::enable_in_range;
use wgpu::CommandEncoder;

use crate::{
    BindGroup, Buffer, BufferInfo, BufferMode, BufferUsage,
    ColorLayout, DepthStencilLayout, GpuPod, Graphics,
    GraphicsPipeline, ModuleKind, PushConstantBitset, PushConstants,
    RenderCommand, SetIndexBufferError, SetPushConstantsError,
    SetVertexBufferError, TriangleBuffer, UniformBuffer, Vertex,
    VertexBuffer,
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
    pub(crate) push_constant_bitset: PushConstantBitset,
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
    ) -> Result<(), SetVertexBufferError> {
        // Check if we can even set the vertex buffer
        let info = self
            .pipeline
            .vertex_config()
            .inputs
            .get(slot as usize)
            .ok_or(SetVertexBufferError::InvalidSlot(slot))?;
        if info.vertex_info() != V::info() {
            return Err(SetVertexBufferError::InvalidVertexInfo(slot));
        }

        // Validate the bounds and convert them to byte bounds
        let (start, end) = convert(bounds, buffer).ok_or(
            SetVertexBufferError::InvalidRange(buffer.len()),
        )?;

        // Store the command within the internal queue
        self.commands.push(RenderCommand::SetVertexBuffer {
            slot,
            buffer: buffer.as_untyped(),
            start,
            end,
        });

        Ok(())
    }

    // Sets the active index buffer with a specific range
    pub fn set_index_buffer(
        &mut self,
        buffer: &'r TriangleBuffer<u32>,
        bounds: impl RangeBounds<usize>,
    ) -> Result<(), SetIndexBufferError> {
        // Validate the bounds and convert them to byte bounds
        let (start, end) = convert(bounds, buffer)
            .ok_or(SetIndexBufferError::InvalidRange(buffer.len()))?;

        // Store the command wtihin the internal queue
        self.commands.push(RenderCommand::SetIndexBuffer {
            buffer: buffer,
            start,
            end,
        });

        Ok(())
    }

    // Set push constants before rendering
    pub fn set_push_constants(
        &mut self,
        callback: impl FnOnce(&mut PushConstants),
    ) -> Result<(), SetPushConstantsError> {
        let shader = self.pipeline.shader();

        // Don't set the push constants if we don't have any to set
        if shader.reflected.push_constant_ranges.is_empty() {
            return Ok(());
        }

        // Make sure we have enough bytes to store the push constants
        let pc = self.push_constant.len()
            - self.push_constant_global_offset;
        if pc < 1024 {
            self.push_constant
                .extend(std::iter::repeat(0).take(1024));
        }

        // Get the max size that we must allocate (at minimum) to be able to use ALL the defined push constants
        let size = shader
            .reflected
            .push_constant_ranges
            .iter()
            .map(|range| range.end - range.start)
            .sum::<u32>() as usize;

        // Get the data that we will use
        let start = self.push_constant_global_offset as usize;
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

        // Create a bitset that contains the bytes that we MUST write to
        let shader = self.pipeline.shader();
        let defined = &shader.reflected().push_constant_bitset;

        // Iterate over all the push constant ranges and create commands
        let mut new_internal_offset: usize = 0;
        for range in push_constants.ranges {
            // Return a warning if the user triest to set push constants that are not defined
            let bits = enable_in_range::<u128>(
                range.start as usize,
                range.end as usize,
            );
            if bits & !defined.set != 0 {
                log::warn!("Trying to set push constants that are not defined, ignoring");
                continue;
            }

            // Create a command to set the push constant
            self.commands.push(RenderCommand::SetPushConstants {
                stages: crate::visibility_to_wgpu_stage(
                    &range.visibility,
                ),
                size: (range.end - range.start) as usize,
                global_offset: self.push_constant_global_offset,
                local_offset: range.start as u32,
            });
            new_internal_offset += size as usize;
        }
        self.push_constant_global_offset += new_internal_offset;
        Ok(())
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
        };

        // Let the user modify the bind group
        callback(&mut bind_group);
        let cache = &self.graphics.0.cached;

        // Extract the resources from bind group (dissociate the lifetime)
        let BindGroup::<'_> {
            reflected,
            resources,
            slots,
            ids,
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
    // TODO: VALIDATION: Make sure all bind groups, push constants, and buffers, have been set
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
    // TODO: VALIDATION: Make sure all bind groups, push constants, and buffers, have been set
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
