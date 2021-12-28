use crate::{RenderTask, is_render_thread, RenderCommandQuery, increment_command_id, RenderCommandQueryResult};
use super::command;

// A batch command result that can call a callback whenever multiple commands have been executed
pub struct BatchRenderCommandQueryResult {
    tasks: Vec<RenderTask>, // The tasks that we will send to the render thread   
}
impl BatchRenderCommandQueryResult {
    // Create a new batch of commands using a vector of query results
    pub fn new(tasks: Vec<RenderTask>) -> Self {
        Self { tasks }
    }
    // We wish to get called whenever all the commands have been executed on the render thread
    pub fn with_callback(self, callback_id: u64) {
        if is_render_thread() { panic!() }
        // Send the commands
        let command_count = self.tasks.len() as u16;
        let thread_id = std::thread::current().id();
        for task in self.tasks {
            let command_id = increment_command_id();
            let query = RenderCommandQuery {
                task,
                callback_id: None,
                batch_callback_data: Some(BatchCallbackData { command_count, callback_id, thread_id  }),
                command_id,
                thread_id,
            };
            command(query);
        }        
    }
}

// Some batch callback data that will be stored in each command query
#[derive(Debug, Clone)]
pub struct BatchCallbackData {
    pub command_count: u16,
    pub callback_id: u64,
    pub thread_id: std::thread::ThreadId,
}