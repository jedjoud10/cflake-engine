use std::cell::RefCell;

// A wrapper around an OpenGL fence, so we can check wether or not some GPU command has finished executing 
pub(crate) struct GlTracker {
    // An OpenGL fence object
    fence: *const gl::types::__GLsync,
    // A callback that we will execute when the fence gets signaled
    callback: RefCell<Option<Box<dyn FnOnce()>>>,
}

impl GlTracker {
    // Create a GlTracker, and call the start function
    // If the OpenGL fence has been signaled, we must run the callback function
    pub fn new<F: FnOnce(), C: FnOnce() + 'static>(start: F, callback_finished: C) -> Self {
        // Create the fence object
        let fence = unsafe {
            // Flush first
            gl::Flush(); 
            // Call the function
            start();
            // Then finally create the fence
            let fence = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
            gl::Flush();
            fence
        };
        Self {
            fence,
            callback: RefCell::new(Some(Box::new(callback_finished))),
        }
    }
    // Check wether the corresponding fence object has completed
    pub fn completed(&self) -> bool {
        let result = unsafe {
            let res = gl::ClientWaitSync(self. fence, gl::SYNC_FLUSH_COMMANDS_BIT, 0);
            // Delete the fence since we won't use it anymore
            gl::DeleteSync(self.fence);
            res
        };        

        // Check
        let completed = result == gl::ALREADY_SIGNALED || result == gl::CONDITION_SATISFIED;
        // If we did complete, we must execute the callback
        if completed {
            let mut callback = self.callback.borrow_mut();
            let callback = callback.take();
            if let Some(callback) = callback { callback(); }
        }

        completed
    }
}