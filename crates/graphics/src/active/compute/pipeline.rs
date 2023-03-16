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
        let shader = self.pipeline.shader();

        // Don't set the push constants if we don't have any to set
        let Some(layout) = shader.reflected.push_constant_layout else {
            return Ok(());
        };

        // Make sure we have enough bytes to store the push constants
        let pc = self.push_constant.len()
            - self.push_constant_global_offset;
        if pc < 1024 {
            self.push_constant
                .extend(std::iter::repeat(0).take(1024));
        }

        // Get the max size that we must allocate (at minimum) to be able to use ALL the defined push constants
        let size = layout.size().get();

        // Get the data that we will use
        let start = self.push_constant_global_offset as usize;
        let end = size as usize + start;
        let data = &mut self.push_constant[start..end];

        // Create push constants that we can set
        let mut push_constants = PushConstants { data, layout };

        // Let the user modify the push constant
        callback(&mut push_constants);

        // Create a command to set the push constant bytes
        match layout {
            // Set the push constants for the compute module
            PushConstantLayout::Single(size, visibility) => {
                assert_eq!(visibility, ModuleVisibility::Compute);
                self.commands.push(
                    ComputeCommand::SetPushConstants {
                        size: size.get() as usize,
                        global_offset: self
                            .push_constant_global_offset,
                        local_offset: 0,
                    },
                );
            }

            // Set the push constants for vertex/fragment modules
            PushConstantLayout::SplitVertexFragment {
                vertex,
                fragment,
            } => {
                panic!()
            }
        }
        self.push_constant_global_offset += size as usize;
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
            .push(ComputeCommand::SetBindGroup(binding, bind_group));
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
