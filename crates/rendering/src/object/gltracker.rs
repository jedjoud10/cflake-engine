use crate::pipeline::Pipeline;

// A callback closure value
pub(crate) type GlTrackerCallback = Option<Box<dyn FnOnce(&Pipeline)>>;

// A wrapper around an OpenGL fence, so we can check wether or not some GPU command has finished executing
pub(crate) struct GlTracker {
    // An OpenGL fence object
    fence: Option<*const gl::types::__GLsync>,
    // A callback that we will execute when the fence gets signaled
    callback: GlTrackerCallback,
}

impl GlTracker {
    // Create a GlTracker, and call the start function
    // If the OpenGL fence has been signaled, we must run the callback function
    pub fn new<F: FnOnce()>(start: F) -> Self {
        // Create the fence object
        let fence = unsafe {
            // Flush first
            gl::Flush();
            // Call the function
            start();
            // Then finally create the fence
            let fence = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
            Some(fence)
        };
        Self { fence, callback: None }
    }
    // Create the tracker with a specific execution callback
    pub fn with_completed_callback<C: FnOnce(&Pipeline) + 'static>(mut self, callback_finished: C) -> Self {
        self.callback = Some(Box::new(callback_finished));
        self
    }

    // Create a GL tracker that will actually execute synchronously, and always be completed if we query it's completed state
    pub fn fake<F: FnOnce()>(start: F) -> Self {
        // Call the function
        start();
        Self { fence: None, callback: None }
    }
    // Check wether the corresponding fence object has completed
    pub fn completed(&mut self, pipeline: &Pipeline) -> bool {
        // Check if this tracker was made to be always completed
        if self.fence.is_none() {
            let callback = self.callback.take();
            if let Some(callback) = callback {
                callback(pipeline);
            }
            return true;
        }
        let result = unsafe { gl::ClientWaitSync(self.fence.unwrap(), gl::SYNC_FLUSH_COMMANDS_BIT, 0) };

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
            let callback = self.callback.take();
            if let Some(callback) = callback {
                callback(pipeline);
            }
        }

        completed
    }
}
