use ahash::AHashMap;
use utils::enable_in_range;
use wgpu::CommandEncoder;

use crate::{
    visibility_to_wgpu_stage, BindGroup, Buffer, BufferInfo,
    BufferMode, BufferUsage, ColorLayout, ComputeCommand,
    ComputePipeline, DepthStencilLayout, Fence, GpuPod, Graphics,
    RenderPipeline, ModuleKind, ModuleVisibility,
    PushConstantLayout, PushConstants, RenderCommand,
    SetIndexBufferError, SetPushConstantsError, SetVertexBufferError,
    TriangleBuffer, UniformBuffer, Vertex, VertexBuffer,
};
use std::{
    collections::hash_map::Entry,
    marker::PhantomData,
    ops::{Bound, Range, RangeBounds},
    sync::Arc,
};

// An active compute pipeline that is bound to a compute pass
pub struct ActiveComputePipeline<'a, 'r> {
    pub(crate) pipeline: &'r ComputePipeline,
    pub(crate) commands: &'a mut Vec<ComputeCommand<'r>>,
    pub(crate) graphics: &'r Graphics,
    pub(crate) push_constant: &'a mut Vec<u8>,
    pub(crate) push_constant_global_offset: usize,
}

impl<'a, 'r> ActiveComputePipeline<'a, 'r> {
    // Set push constants before dispatching a compute call
    pub fn set_push_constants(
        &mut self,
        callback: impl FnOnce(&mut PushConstants),
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
            // Set the push constants for the compute module
            PushConstantLayout::Single(size, ModuleVisibility::Compute) => {
                self.commands.push(
                    ComputeCommand::SetPushConstants {
                        size: size.get() as usize,
                        global_offset: copied_push_constant_global_offset,
                        local_offset: 0,
                    },
                );
            }

            // Should not be possible
            _ => panic!(),
        }

        Ok(())
    }

    // Execute a callback that we will use to fill a bind group
    pub fn set_bind_group<'b>(
        &mut self,
        binding: u32,
        callback: impl FnOnce(&mut BindGroup<'b>),
    ) {
        if let Some(bind_group) = super::create_bind_group(
            self.graphics,
            self.pipeline.shader().reflected.clone(),
            binding,
            callback
        ) {
            self.commands
                .push(ComputeCommand::SetBindGroup(binding, bind_group));
        }
    }

    // Executed before any dispatch calls to make sure that we have
    // all the necessities (bind groups, push constants) to be able to dispatch
    pub fn validate(&self) {
        // TODO: VALIDATION: Make sure all bind groups, push constants, have been set
    }

    // Execute the current compute shader call
    // TODO: Handle fence shenanigans
    pub fn dispatch(&mut self, size: vek::Vec3<u32>) {
        self.validate();
        self.commands.push(ComputeCommand::Dispatch {
            x: size.x,
            y: size.y,
            z: size.z,
        });
    }

    // Get the underlying compute pipeline that is currently bound
    pub fn pipeline(&self) -> &ComputePipeline {
        self.pipeline
    }
}
