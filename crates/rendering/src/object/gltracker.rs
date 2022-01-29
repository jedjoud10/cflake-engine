use std::cell::RefCell;

use crate::pipeline::Pipeline;

// A wrapper around an OpenGL fence, so we can check wether or not some GPU command has finished executing
pub(crate) struct GlTracker {
    // An OpenGL fence object
    fence: Option<*const gl::types::__GLsync>,
    // A callback that we will execute when the fence gets signaled
    callback: RefCell<Option<Box<dyn FnOnce(&Pipeline)>>>,
}

impl GlTracker {
    // Create a GlTracker, and call the start function
    // If the OpenGL fence has been signaled, we must run the callback function
    pub fn new<F: FnOnce(&Pipeline), C: FnOnce(&Pipeline) + 'static>(start: F, callback_finished: C, pipeline: &Pipeline) -> Self {
        // Create the fence object
        let fence = unsafe {
            // Flush first
            gl::Flush();
            // Call the function
            start(pipeline);
            // Then finally create the fence
            let fence = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
            gl::Flush();
            Some(fence)
        };
        Self {
            fence,
            callback: RefCell::new(Some(Box::new(callback_finished))),
        }
    }
    // Create a GL tracker that will actually execute synchronously, and always be completed if we query it's completed state
    pub fn fake<F: FnOnce(&Pipeline)>(start: F, pipeline: &Pipeline) -> Self {
        unsafe {
            // Flush first
            gl::Flush();
            // Call the function
            start(pipeline);
            gl::Finish();
        };
        Self {
            fence: None,
            callback: RefCell::new(None),
        }
    }
    // Check wether the corresponding fence object has completed
    pub fn completed(&self, pipeline: &Pipeline) -> bool {
        // Check if this tracker was made to be always completed
        if self.fence.is_none() { return true; }
        let result = unsafe {
            let res = gl::ClientWaitSync(self.fence.unwrap(), gl::SYNC_FLUSH_COMMANDS_BIT, 0);
            res
        };

        // Check
        let completed = result == gl::ALREADY_SIGNALED || result == gl::CONDITION_SATISFIED;
        if completed {
            // Delete the fence since we won't use it anymore
            unsafe {
                gl::DeleteSync(self.fence.unwrap());
            }
        }
        // If we did complete, we must execute the callback
        if completed {
            let mut callback = self.callback.borrow_mut();
            let callback = callback.take();
            if let Some(callback) = callback {
                callback(pipeline);
            }
        }

        completed
    }
}
