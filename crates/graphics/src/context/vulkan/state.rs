use super::Command;
use crate::{Access, InsertVkCommand};
use ash::vk;

// Recorder state that is stored within the recorders that is dynamically bound to command buffers
#[derive(Default)]
pub(crate) struct State {
    pub(crate) commands: Vec<Command>,
    pub(crate) access: Vec<Access>,
}

// A finished command buffer state is what allows us to directly record Vulkan commands
pub(crate) type CompletedState = Vec<Group>;

// A single group an the barrier that comes after it
pub(crate) struct Group {
    pub(crate) commands: Vec<Command>,
    pub(crate) barrier: Option<Barrier>,
}

// Command pipeline barrier abstraction
// This helps automatically synchronizing vulkan commands
pub(crate) struct Barrier {
    pub src_stage_mask: vk::PipelineStageFlags,
    pub dst_stage_mask: vk::PipelineStageFlags,
    pub dependency_flags: vk::DependencyFlags,
    pub memory_barriers: Vec<vk::MemoryBarrier>,
    pub buffer_memory_barriers: Vec<vk::BufferMemoryBarrier>,
    pub image_memory_barriers: Vec<vk::ImageMemoryBarrier>,
}

impl InsertVkCommand for Barrier {
    unsafe fn insert(
        self,
        device: &ash::Device,
        buffer: vk::CommandBuffer,
    ) {
        device.cmd_pipeline_barrier(
            buffer,
            self.src_stage_mask,
            self.dst_stage_mask,
            self.dependency_flags,
            &self.memory_barriers,
            &self.buffer_memory_barriers,
            &self.image_memory_barriers,
        );
    }
}

// Convert the locally stored command to local groups that automatically place barriers within them
pub(crate) fn complete(_state: State) -> CompletedState {
    todo!()
}
