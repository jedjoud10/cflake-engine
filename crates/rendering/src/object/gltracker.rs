// A wrapper around an OpenGL fence, so we can check wether or not some GPU command has finished executing 
pub(crate) struct GlTracker {
    // An OpenGL fence object
    fence: *const gl::types::__GLsync,
}

unsafe impl Sync for GlTracker {}
unsafe impl Send for GlTracker {} 

impl GlTracker {
    // Create a GlTracker, and call the function
    pub fn new<F: FnOnce()>(inner: F) -> Self {
        // Create the fence object
        let fence = unsafe {
            // Flush first
            gl::Flush(); 
            // Call the function
            inner();
            // Then finally create the fence
            let fence = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
            gl::Flush();
            fence
        };
        Self {
            fence
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
        result == gl::ALREADY_SIGNALED || result == gl::CONDITION_SATISFIED
    }
}