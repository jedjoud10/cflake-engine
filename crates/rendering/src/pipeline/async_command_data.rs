use super::{batch_command::BatchCallbackData, buffer::PipelineBuffer};
use crate::{compute::ComputeShaderSubTasks, GPUObjectID};
#[derive(Clone)]
pub struct InternalAsyncGPUCommandData {
    pub command_id: u64,
    pub callback_id: Option<(u64, std::thread::ThreadId)>,
    pub batch_callback_data: Option<BatchCallbackData>,
}

// Some events that we will execute after we finish executing the async command
pub enum AsyncGPUCommandExecutionEvent {
    ComputeShaderSubTasks(GPUObjectID, ComputeShaderSubTasks),
}

pub struct AsyncGPUCommandData {
    pub sync: *const gl::types::__GLsync, // The OpenGL fence sync
    pub internal: Option<InternalAsyncGPUCommandData>,
    pub execution_event: Option<AsyncGPUCommandExecutionEvent>,
}

impl AsyncGPUCommandData {
    // New
    pub fn new(execution_event: Option<AsyncGPUCommandExecutionEvent>) -> Self {
        let sync = unsafe { gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0) };
        Self {
            sync,
            internal: None,
            execution_event,
        }
    }
    // Give the data a bit more information about the command that created it
    pub fn additional_command_data(&mut self, command_data: InternalAsyncGPUCommandData) {
        self.internal = Some(command_data);
    }
    // Check if the corresponding GPU data of the fence has executed on the GPU
    pub fn has_executed(&self) -> bool {
        unsafe {
            let result = gl::ClientWaitSync(self.sync, gl::SYNC_FLUSH_COMMANDS_BIT, 0);
            if result == gl::CONDITION_SATISFIED || result == gl::ALREADY_SIGNALED {
                // The sync was signaled, meaning the async command was executed
                true
            } else {
                false
            }
        }
    }
    // Execute the execution event
    pub fn execute_event(&mut self, buf: &PipelineBuffer) {
        if let Option::Some(event) = self.execution_event.take() {
            match event {
                AsyncGPUCommandExecutionEvent::ComputeShaderSubTasks(id, compute_tasks) => compute_tasks.run(buf, id),
            }
        }
    }
}
