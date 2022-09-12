use std::time::Duration;
use crate::context::Context;

// This is an abstraction over an OpenGL fence that we can use to detect whenever the GPU has completed executing some commands
// TODO: Actually figure out how to use these bozos (asnyc buffer transfer and/or compute shaders)
pub struct Fence {
    fence: Option<gl::types::GLsync>,
    active: bool,
}

impl Fence {
    // Create a new command timer that we can use to check if specific GPU commands have finished executing
    pub fn new(ctx: &mut Context) -> Self {
        Self {
            fence: None,
            active: false,
        }
    }

    // Start the command timer query and start queueing up commands internally
    pub fn start(&mut self) {
        if !self.active {
            unsafe {
                gl::Flush();
                self.active = true;
            }
        }
    }

    // Stop the command timer and stop queuing up commands internally
    pub fn stop(&mut self) {
        if self.active {
            unsafe {
                let fence = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
                self.fence = Some(fence);
                self.active = false;
            }
        }
    }

    // Check if the OpenGL fence has been signaled
    pub fn signaled(&self) -> bool {
        if self.fence.is_none() || self.active {
            return false;
        }

        let state = unsafe { gl::ClientWaitSync(self.fence.unwrap(),  gl::SYNC_FLUSH_COMMANDS_BIT, 0) };
        match state {
            gl::ALREADY_SIGNALED | gl::CONDITION_SATISFIED => true,
            gl::WAIT_FAILED | gl::TIMEOUT_EXPIRED => false,
            _ => false,
        }
    }
}
