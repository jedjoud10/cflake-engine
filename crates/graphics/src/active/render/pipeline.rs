use ahash::AHashMap;
use utils::enable_in_range;
use wgpu::CommandEncoder;

use crate::{
    active::pipeline::ActivePipeline, visibility_to_wgpu_stage, BindGroup, Buffer, BufferInfo,
    BufferMode, BufferUsage, ColorLayout, DepthStencilLayout, DrawError, DrawIndexedError,
    DrawIndexedIndirectBuffer, DrawIndirectBuffer, GpuPod, Graphics, ModuleKind, ModuleVisibility,
    PushConstantLayout, PushConstants, RenderCommand, RenderPipeline, SetBindGroupError,
    SetIndexBufferError, SetPushConstantsError, SetVertexBufferError, TriangleBuffer,
    UniformBuffer, Vertex, VertexBuffer,
};
use std::{
    collections::hash_map::Entry,
    marker::PhantomData,
    ops::{Bound, Range, RangeBounds},
    sync::Arc,
};

// An active graphics pipeline that is bound to a render pass that we can use to render
pub struct ActiveRenderPipeline<'a, 'r, 't, C: ColorLayout, DS: DepthStencilLayout> {
    pub(crate) pipeline: &'r RenderPipeline<C, DS>,
    pub(crate) commands: &'a mut Vec<RenderCommand<'r, C, DS>>,
    pub(crate) graphics: &'r Graphics,
    pub(crate) push_constant: &'a mut Vec<u8>,
    pub(crate) push_constant_global_offset: usize,
    pub(crate) _phantom: PhantomData<&'t C>,
    pub(crate) set_groups_bitflags: u32,
    pub(crate) set_vertex_buffer_slots: u32,
    pub(crate) set_index_buffer: bool,
    pub(crate) reflected_groups_bitflags: u32,
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

impl<'a, 'r, 't, C: ColorLayout, DS: DepthStencilLayout> ActiveRenderPipeline<'a, 'r, 't, C, DS> {
    // Assign a vertex buffer to a slot with a specific range
    pub fn set_vertex_buffer<V: Vertex>(
        &mut self,
        slot: u32,
        buffer: &'r VertexBuffer<V>,
        bounds: impl RangeBounds<usize>,
    ) -> Result<(), SetVertexBufferError> {
        self.set_vertex_buffer_slots &= !(1 << slot);

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
        let (start, end) =
            convert(bounds, buffer).ok_or(SetVertexBufferError::InvalidRange(buffer.len()))?;

        // Store the command within the internal queue
        self.commands.push(RenderCommand::SetVertexBuffer {
            slot,
            buffer: buffer.as_untyped(),
            start,
            end,
        });

        self.set_vertex_buffer_slots |= 1 << slot;

        Ok(())
    }

    // Sets the active index buffer with a specific range
    pub fn set_index_buffer(
        &mut self,
        buffer: &'r TriangleBuffer<u32>,
        bounds: impl RangeBounds<usize>,
    ) -> Result<(), SetIndexBufferError> {
        // Validate the bounds and convert them to byte bounds
        self.set_index_buffer = false;
        let (start, end) =
            convert(bounds, buffer).ok_or(SetIndexBufferError::InvalidRange(buffer.len()))?;

        // Store the command wtihin the internal queue
        self.commands.push(RenderCommand::SetIndexBuffer {
            buffer: buffer,
            start,
            end,
        });

        self.set_index_buffer = true;
        Ok(())
    }

    // Check if the user set the required vertex buffer
    // Returns Ok if the buffers have been set properly
    fn validate_vertex_buffers(&self) -> Result<(), u32> {
        let needed = self.pipeline.vertex_config().inputs.len() as u32;
        let set = self.set_vertex_buffer_slots.count_ones();
        if set != needed {
            let first = self.set_vertex_buffer_slots.leading_zeros();
            Err(first)
        } else {
            Ok(())
        }
    }

    // Check if the user set the required index buffer
    // Returns true if the buffer is validated
    fn validate_index_buffers(&self) -> bool {
        self.set_index_buffer
    }

    // Draw a number of primitives using the currently bound vertex buffers
    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) -> Result<(), DrawError> {
        // Handle the missing bind groups
        if let Err(value) =
            crate::validate_set(self.reflected_groups_bitflags, self.set_groups_bitflags)
        {
            return Err(DrawError::MissingValidBindGroup(value));
        }

        // Check for missing vertex buffers
        if let Err(slot) = self.validate_vertex_buffers() {
            return Err(DrawError::MissingVertexBuffer(slot));
        }

        self.commands.push(RenderCommand::Draw {
            vertices,
            instances,
        });

        Ok(())
    }

    // Draw a number of primitives using the currently bound vertex buffers and the given draw indirect buffer
    pub fn draw_indirect(
        &mut self,
        buffer: &'r DrawIndirectBuffer,
        element: usize,
    ) -> Result<(), DrawError> {
        // Handle the missing bind groups
        if let Err(value) =
            crate::validate_set(self.reflected_groups_bitflags, self.set_groups_bitflags)
        {
            return Err(DrawError::MissingValidBindGroup(value));
        }

        // Check for missing vertex buffers
        if let Err(slot) = self.validate_vertex_buffers() {
            return Err(DrawError::MissingVertexBuffer(slot));
        }

        // Check if the element index is ok
        if element >= buffer.len() {
            return Err(DrawError::InvalidIndirectIndex);
        }

        self.commands
            .push(RenderCommand::DrawIndirect { buffer, element });

        Ok(())
    }

    // Dispatch multiple draw calls that will draw a number of vertices using the currently bound vertex buffers
    pub fn multi_draw_indirect(
        &mut self,
        buffer: &'r DrawIndirectBuffer,
        offset: usize,
        count: usize,
    ) -> Result<(), DrawError> {
        // Handle the missing bind groups
        if let Err(value) =
            crate::validate_set(self.reflected_groups_bitflags, self.set_groups_bitflags)
        {
            return Err(DrawError::MissingValidBindGroup(value));
        }

        // Check for missing vertex buffers
        if let Err(slot) = self.validate_vertex_buffers() {
            return Err(DrawError::MissingVertexBuffer(slot));
        }

        // Check if the element index is ok
        if offset + count >= buffer.len() {
            return Err(DrawError::InvalidIndirectIndex);
        }

        self.commands
            .push(RenderCommand::MultiDrawIndirect { buffer, offset, count });

        Ok(())
    }

    // Draw a number of indexed primitives using the currently bound vertex buffers and index buffer
    pub fn draw_indexed(
        &mut self,
        indices: Range<u32>,
        instances: Range<u32>,
    ) -> Result<(), DrawIndexedError> {
        // Handle the missing bind groups
        if let Err(value) =
            crate::validate_set(self.reflected_groups_bitflags, self.set_groups_bitflags)
        {
            return Err(DrawIndexedError::MissingValidBindGroup(value));
        }

        // Check for missing vertex buffers
        if let Err(slot) = self.validate_vertex_buffers() {
            return Err(DrawIndexedError::MissingVertexBuffer(slot));
        }

        // Check for missing index buffers
        if !self.validate_index_buffers() {
            return Err(DrawIndexedError::MissingIndexBuffer);
        }

        self.commands
            .push(RenderCommand::DrawIndexed { indices, instances });

        Ok(())
    }

    // Draw a number of indexed primitives using the currently bound vertex buffers, index buffer, and draw indexed indirect buffer
    pub fn draw_indexed_indirect(
        &mut self,
        buffer: &'r DrawIndexedIndirectBuffer,
        element: usize,
    ) -> Result<(), DrawIndexedError> {
        // Handle the missing bind groups
        if let Err(value) =
            crate::validate_set(self.reflected_groups_bitflags, self.set_groups_bitflags)
        {
            return Err(DrawIndexedError::MissingValidBindGroup(value));
        }

        // Check for missing vertex buffers
        if let Err(slot) = self.validate_vertex_buffers() {
            return Err(DrawIndexedError::MissingVertexBuffer(slot));
        }

        // Check for missing index buffers
        if !self.validate_index_buffers() {
            return Err(DrawIndexedError::MissingIndexBuffer);
        }

        // Check if the element index is ok
        if element >= buffer.len() {
            return Err(DrawIndexedError::InvalidIndirectIndex);
        }

        self.commands
            .push(RenderCommand::DrawIndexedIndirect { buffer, element });

        Ok(())
    }

    // Dispatch multiple draw calls that will draw a number of indexed primitives using the currently bound vertex buffers
    pub fn multi_draw_indexed_indirect(
        &mut self,
        buffer: &'r DrawIndexedIndirectBuffer,
        offset: usize,
        count: usize,
    ) -> Result<(), DrawIndexedError> {
        // Handle the missing bind groups
        if let Err(value) =
            crate::validate_set(self.reflected_groups_bitflags, self.set_groups_bitflags)
        {
            return Err(DrawIndexedError::MissingValidBindGroup(value));
        }

        // Check for missing vertex buffers
        if let Err(slot) = self.validate_vertex_buffers() {
            return Err(DrawIndexedError::MissingVertexBuffer(slot));
        }

        // Check for missing index buffers
        if !self.validate_index_buffers() {
            return Err(DrawIndexedError::MissingIndexBuffer);
        }

        // Check if the element index is ok
        if offset + count >= buffer.len() {
            return Err(DrawIndexedError::InvalidIndirectIndex);
        }

        self.commands
            .push(RenderCommand::MultiDrawIndexedIndirect { buffer, offset, count });

        Ok(())
    }
}

impl<'a, 'r, 't, C: ColorLayout, DS: DepthStencilLayout> ActivePipeline
    for ActiveRenderPipeline<'a, 'r, 't, C, DS>
{
    type Pipeline = &'r RenderPipeline<C, DS>;

    // Set push constants before rendering
    fn set_push_constants(
        &mut self,
        callback: impl FnOnce(&mut PushConstants<Self>),
    ) -> Result<(), SetPushConstantsError> {
        // Get the push constant layout used by the shader
        // and push new bytes onto the internally stored constants
        let copied_push_constant_global_offset = self.push_constant_global_offset;
        let Some(layout) = super::handle_push_constants(
            self.pipeline.shader().reflected.clone(),
            &mut self.push_constant,
            &mut self.push_constant_global_offset,
            callback
        ) else { return Ok(()) };

        // Create a command to set the push constant bytes
        match layout {
            // Set the push constants for SharedVG or Vert/Frag modules
            PushConstantLayout::Single(size, visibility) => {
                self.commands.push(RenderCommand::SetPushConstants {
                    stages: visibility_to_wgpu_stage(&visibility),
                    size: size.get() as usize,
                    global_offset: copied_push_constant_global_offset,
                    local_offset: 0,
                });
            }

            // Set the push constants for vertex/fragment modules
            PushConstantLayout::SplitVertexFragment { vertex, fragment } => {
                // Set the vertex push constants if its bytes are defined
                self.commands.push(RenderCommand::SetPushConstants {
                    stages: wgpu::ShaderStages::VERTEX,
                    size: vertex.get() as usize,
                    global_offset: copied_push_constant_global_offset,
                    local_offset: 0,
                });

                // Set the fragment push constants if its bytes are defined
                self.commands.push(RenderCommand::SetPushConstants {
                    stages: wgpu::ShaderStages::FRAGMENT,
                    size: fragment.get() as usize,
                    global_offset: copied_push_constant_global_offset + vertex.get() as usize,
                    local_offset: vertex.get() as usize,
                });
            }
        }

        Ok(())
    }

    // Execute a callback that we will use to fill a bind group
    fn set_bind_group<'b>(
        &mut self,
        binding: u32,
        callback: impl FnOnce(&mut BindGroup<'b>),
    ) -> Result<(), SetBindGroupError> {
        let shader = self.pipeline.shader();
        self.set_groups_bitflags &= !(1 << binding);
        if let Some(bind_group) = super::create_bind_group(
            self.graphics,
            &[shader.vertex().name(), shader.fragment().name()],
            self.pipeline.shader().reflected.clone(),
            binding,
            callback,
        )? {
            self.commands
                .push(RenderCommand::SetBindGroup(binding, bind_group));
        }

        self.set_groups_bitflags |= 1 << binding;

        Ok(())
    }

    // Get the underlying graphics pipeline that is currently bound
    fn inner(&self) -> Self::Pipeline {
        self.pipeline
    }
}
