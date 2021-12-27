pub struct AsyncGPUCommandData {    
    pub sync: *const gl::types::__GLsync, // The OpenGL fence sync
    pub command_data: Option<(u64, Option<(u64, std::thread::ThreadId)>)>,
}

impl AsyncGPUCommandData {
    // New
    pub fn new(sync: *const gl::types::__GLsync) -> Self {
        Self {
            sync,
            command_data: None,
        }
    }
    // Check if the corresponding GPU data of the fence has executed on the GPU
    pub fn has_executed(&self) -> bool {
        unsafe {
            let result = gl::ClientWaitSync(self.sync, gl::SYNC_FLUSH_COMMANDS_BIT, 0);
            if result == gl::CONDITION_SATISFIED || result == gl::ALREADY_SIGNALED {
                // The sync was signaled, meaning the async command was executed
                true
            } else { false }
        }
    }
}