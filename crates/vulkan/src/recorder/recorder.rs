use std::sync::Arc;

use crate::{Device, Pool, State};
use ash::vk;
use parking_lot::{Mutex, MutexGuard};
use utils::{BitSet, SharedVec};

// A recorder is a command buffer that is currently recording commands
// Recorders will automatically put semaphores and fences when necessary
// This will keep track of the operations done on specific buffers and
// automatically put semaphores between operations that affect the same object
pub struct Recorder<'d> {
    // Raw command buffer and it's idnex
    pub(crate) index: usize,
    pub(crate) cmd: vk::CommandBuffer,
    pub(crate) info: vk::CommandBufferBeginInfo,

    // List of command buffers that are currently being recorded
    pub(crate) free: Arc<Mutex<BitSet>>,

    // Data related to context
    pub(crate) state: Arc<Mutex<Vec<State>>>,
    pub(crate) device: &'d Device,
    pub(crate) pool: &'d Pool,
}

impl<'d> Recorder<'d> {
    // Add a new command to the recorder
    unsafe fn push(&mut self, cmd: super::Command) {
        let mut locked = self.state.lock();
        let state = &mut locked[self.index];
        state.0.push(cmd);
    }

    // Convert the CPU commands to actual vulkan commands and write them to the buffer
    pub unsafe fn finish(mut self) -> vk::CommandBuffer {
        let mut locked = self.state.lock();
        let state = &mut locked[self.index];
        let taken = std::mem::take(&mut *state);
        taken.finish(self.cmd, self.info, self.device);
        self.free.lock().remove(self.index);
        self.cmd
    }
}

// Buffer commands
impl<'d> Recorder<'d> {
    // Copy a buffer region to another buffer
    pub unsafe fn cmd_copy_buffer(
        &mut self,
        src: vk::Buffer,
        dst: vk::Buffer,
        size: u64,
        src_offset: u64,
        dst_offset: u64,
    ) {
        let region = vk::BufferCopy {
            src_offset,
            dst_offset,
            size,
        };

        self.push(super::Command::BufferCopy {
            src,
            dst,
            regions: vec![region],
        });
    }

    // Clear the buffer "src"
    pub unsafe fn cmd_clear_buffer(
        &mut self,
        src: vk::Buffer,
        offset: u64,
        size: u64,
    ) {
        self.push(super::Command::BufferFill {
            src,
            offset,
            size,
            data: 0,
        });
    }
}
