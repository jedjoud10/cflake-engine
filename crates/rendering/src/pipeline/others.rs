use std::{sync::{Mutex, atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering}}, collections::HashMap};
use crate::{GPUObjectID};
use lazy_static::lazy_static;

// We update this atomic data whenever we execute a command, and return it's optional GPUObjectID data
#[derive(Default)]
struct AtomicCommandExecutionResult {
    valid: AtomicBool, // Is the result valid? If it is, than we can read it and check equality. If it was not, we wait until it becomes valid
    command_id: AtomicU64,
    id_opt: AtomicUsize,
}

lazy_static! {
    // A global buffer that contains a conversion HashMap of Names -> GPUObjectIDs
    pub(crate) static ref GLOBAL_BUFFER: Mutex<GlobalBuffer> = Mutex::new(GlobalBuffer::default()); 
    static ref RESULT: AtomicCommandExecutionResult = AtomicCommandExecutionResult::default();
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
        if RESULT.valid.load(Ordering::Relaxed) {
            // The result is valid, we can check for equality
            if RESULT.command_id.load(Ordering::Relaxed) == command_id {
                // We can now create a copy of the GPUObjectID
                let id = GPUObjectID { index: RESULT.id_opt.load(Ordering::Relaxed) };
                RESULT.valid.store(false, Ordering::Relaxed);
                return id;
            }
        }
    }
}
// Wait for the execution of a specific command
pub fn wait_execution(command_id: u64) {
    loop {
        if RESULT.valid.load(Ordering::Relaxed) {
            // The result is valid, we can check for equality
            if RESULT.command_id.load(Ordering::Relaxed) == command_id {
                // We got acknowledgment that we executed the command on the render thread!
                RESULT.valid.store(false, Ordering::Relaxed);
                return;
            }
        }
    }
}
// We have executed a command, possibly with a returned GPU ID
pub fn executed_command(command_id: u64, id_opt: Option<GPUObjectID>) {
    // Update the atomics
    if let Option::Some(id) = id_opt { RESULT.id_opt.store(id.index, Ordering::Relaxed); }
    RESULT.command_id.store(command_id, Ordering::Relaxed);
    RESULT.valid.store(true, Ordering::Relaxed)
}