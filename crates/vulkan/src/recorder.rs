use std::time::{Duration, Instant};

use crate::{Device, CommandPool};

use super::State;
use ash::vk;

// A recorder can keep a command buffer cached until we flush it
// This is used to reduce the number of submissions we have to make to the GPU
pub struct Recorder {
    // Data used internally by the abstraction layer
    pub(super) index: usize,
    pub(super) pool: usize,
    pub(super) state: State,

    // Raw command buffer
    pub(super) raw_command_buffer: vk::CommandBuffer,
    pub(super) raw_command_pool: vk::CommandPool,
}

impl Recorder {
    // Get the raw command buffer used by this recorder
    // Note: The command buffer isn't currently recording
    pub fn raw_command_buffer(&self) -> vk::CommandBuffer {
        self.raw_command_buffer
    }

    // Get the raw command pool that allocated the cmd buffer
    pub fn raw_command_pool(&self) -> vk::CommandPool {
        self.raw_command_pool
    }
}

// This is a submission of a command recorder
// The underlying command buffer might've not been submitted yet
pub struct Submission<'a> {
    pub(crate) index: usize,
    pub(crate) queue: vk::Queue,
    pub(crate) pool: &'a CommandPool,
    pub(crate) device: &'a Device,
    pub(crate) flushed: bool,
    pub(crate) force: bool,
}

impl<'a> Submission<'a> {    
    // Wait until the submission completes, and return the elapsedtime
    pub fn wait(mut self) -> Duration {        
        let i = Instant::now();
        self.wait_internal();
        i.elapsed()
    }

    // We *can* take a reference instead of consuming the type
    fn wait_internal(&mut self) {
        // Flush the submission and start executing it on the GPU
        log::debug!("Waiting for submission {} from queue {:?}", self.index, self.queue);
        let fence = unsafe { self.pool.flush_specific(self.queue, self.device, self.index, true) };
        log::debug!("Waiting on fence {:?}...", fence);
        
        // Wait for the fence (if we have one) to complete
        if let Some(fence) = fence {
            unsafe { self.device.raw().wait_for_fences(&[fence], true, u64::MAX).unwrap() };
        } else {
            log::warn!("Waiting on submission that doesn't have a fence!");
        }
        self.flushed = true;
    }

    /*
    // Force an immediate flush of the buffer
    pub fn flush(mut self) {
        log::debug!("Flusing submission {} from queue {:?}", self.index, self.queue);
        unsafe { self.pool.flush_specific(self.queue, self.device, self.index, false) };
        self.flushed = true;
    }
    */
}

impl<'a> Drop for Submission<'a> {
    fn drop(&mut self) {
        if !self.flushed {
            self.wait_internal();
        }
    }
}