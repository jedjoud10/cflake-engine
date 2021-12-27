use crate::GPUObjectID;

use super::buffer::PipelineBuffer;

pub struct AsyncGPUCommandData {    
    pub sync: *const gl::types::__GLsync, // The OpenGL fence sync
    pub command_data: Option<(u64, Option<(u64, std::thread::ThreadId)>)>,
    pub gpuobjectid_opt: Option<GPUObjectID>,
    pub callback: fn(GPUObjectID, &mut PipelineBuffer),
}

impl AsyncGPUCommandData {
    // New
    pub fn new(sync: *const gl::types::__GLsync, gpuobjectid_opt: Option<GPUObjectID>, callback: fn(GPUObjectID, &mut PipelineBuffer)) -> Self {
        Self {
            sync,
            gpuobjectid_opt,
            command_data: None,
            callback
        }
    }
    // Give the data a bit more information about the command that created it
    pub fn additional_command_data(&mut self, command_data: (u64, Option<(u64, std::thread::ThreadId)>)) {
        self.command_data = Some(command_data);
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