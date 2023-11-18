use ahash::AHashMap;
use utils::enable_in_range;
use wgpu::CommandEncoder;

use crate::{
    active::pipeline::ActivePipeline, visibility_to_wgpu_stage, BindGroup, Buffer, BufferInfo,
    BufferMode, BufferUsage, ColorLayout, ComputeCommand, ComputeShader, DepthStencilLayout,
    DispatchError, GpuPod, Graphics, ModuleVisibility, PushConstantLayout, PushConstants,
    SetBindGroupError, SetPushConstantsError,
};
use std::{
    collections::hash_map::Entry,
    marker::PhantomData,
    ops::{Bound, Range, RangeBounds},
    sync::Arc,
};

// An active compute pipeline that is bound to a compute pass
pub struct ActiveComputeDispatcher<'a, 'r> {
}

impl<'a, 'r> ActiveComputeDispatcher<'a, 'r> {
    // Dispatch the current compute shader using the specified size
    // Executed before any dispatch calls to make sure that we have
    // all the necessities (bind groups, push constants) to be able to dispatch
    pub fn dispatch(&mut self, size: vek::Vec3<u32>) -> Result<(), DispatchError> {
        todo!()
    }
}

impl<'a, 'r> ActivePipeline for ActiveComputeDispatcher<'a, 'r> {
    type Pipeline = &'r ComputeShader;

    // Set push constants before dispatching a compute call
    fn set_push_constants(
        &mut self,
        callback: impl FnOnce(&mut PushConstants<Self>),
    ) -> Result<(), SetPushConstantsError> {
        todo!()
    }

    // Execute a callback that we will use to fill a bind group
    fn set_bind_group<'b>(
        &mut self,
        binding: u32,
        group: BindGroup<'b>
    ) -> Result<(), SetBindGroupError> {
        todo!()
    }

    // Get the underlying compute shader that is currently bound
    fn inner(&self) -> Self::Pipeline {
        todo!()
    }
}
