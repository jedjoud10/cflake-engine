use crate::GPUObjectID;
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        Mutex,
    },
};

use super::{batch_command::BatchCallbackData, buffer::PipelineBuffer};

#[derive(Default)]
pub struct CommandExecutionResults {
    pub results: HashMap<u64, Option<GPUObjectID>>,
}

lazy_static! {
    // A global buffer that contains a conversion HashMap of Names -> GPUObjectIDs
    pub(crate) static ref GLOBAL_BUFFER: Mutex<GlobalBuffer> = Mutex::new(GlobalBuffer::default());
    pub(crate) static ref RESULT: Mutex<CommandExecutionResults> = Mutex::new(CommandExecutionResults::default());
}

// A global buffer that will be accessible by each worker thread
#[derive(Default)]
pub struct GlobalBuffer {
    pub names_to_id: HashMap<String, GPUObjectID>,
}

impl GlobalBuffer {
    // Add an ID with it's corresponding name
    pub fn add_id(&mut self, name: &str, id: GPUObjectID) {
        self.names_to_id.insert(name.to_string(), id);
    }
    // Get if a GPU object name is present in the global buffer
    pub fn gpuobject_name_valid(&self, name: &str) -> bool {
        self.names_to_id.contains_key(name)
    }
    // Get the ID of a GPU object name from within the buffer
    pub fn get_id(&self, name: &str) -> Option<GPUObjectID> {
        let id = self.names_to_id.get(name)?;
        Some(id.clone())
    }
}
// Add an ID with it's corresponding name
pub fn add_id(name: &str, id: GPUObjectID) {
    let mut buf = GLOBAL_BUFFER.lock().unwrap();
    buf.add_id(name, id);
}
// Get if a GPU object name is present in the global buffer
pub fn gpuobject_name_valid(name: &str) -> bool {
    let buf = GLOBAL_BUFFER.lock().unwrap();
    buf.gpuobject_name_valid(name)
}
// Get the ID of a GPU object name from within the buffer
pub fn get_id(name: &str) -> Option<GPUObjectID> {
    let buf = GLOBAL_BUFFER.lock().unwrap();
    buf.get_id(name)
}
// Wait for the creation of a GPU object using a command ID
pub fn wait_id(command_id: u64) -> GPUObjectID {
    loop {
        // Check for equality
        let mut lock = RESULT.lock().unwrap();
        if lock.results.contains_key(&command_id) {
            // We can now create a copy of the GPUObjectID
            let x = lock.results.remove(&command_id).flatten().unwrap();
            return x;
        }
    }
}
// Wait for the execution of a specific command
pub fn wait_execution(command_id: u64) {
    loop {
        // Check for equality
        let mut lock = RESULT.lock().unwrap();
        if lock.results.contains_key(&command_id) {
            // We can now create a copy of the GPUObjectID
            lock.results.remove(&command_id);
            return;
        }
    }
}
// We have executed a command, possibly with a returned GPU ID
pub fn executed_command(buf: &mut PipelineBuffer, command_id: u64, batch_callback_data: Option<BatchCallbackData>, id_opt: Option<GPUObjectID>, x: &mut CommandExecutionResults) {
    // Update the mutex
    // We can now create a copy of the GPUObjectID
    x.results.insert(command_id, id_opt);

    // If we have received a command that was sent in a batch, we must handle the command execution count
    if let Option::Some(batch_callback_data) = batch_callback_data {
        let callback_id = batch_callback_data.callback_id;
        let count = buf.batch_commands_executed.entry(callback_id).or_insert(batch_callback_data.command_count);
        // We must decrement the counter every time we execute a command
        *count -= 1;

        // If the counter reaches 0, we have executed all the commands that we need, so we can call the callback now
        if *count == 0 {
            buf.callback_objects.insert(callback_id, (None, batch_callback_data.thread_id));
            // Also remove it
            buf.batch_commands_executed.remove(&callback_id);
        }
    }
}
