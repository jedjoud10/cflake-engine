use std::collections::{HashMap, HashSet};
use others::SmartList;

use crate::GPUObject;

// A simple Buffer containing the GPU objects that have been generated on the pipeline thread
#[derive(Default)]
pub struct GPUObjectBuffer {    
    pub gpuobjects: SmartList<GPUObject>, // GPU objects
    pub waitable_objects: HashMap<u64, usize>, // Waitable ID to GPUObject index
    pub callback_objects: HashMap<u64, (usize, std::thread::ThreadId)>, // Callback ID to GPUObject index
    pub names_to_id: HashMap<String, usize>, // Names to GPUObject index

    // This is for tasks that have executed but that do not have a an associated GPU object
    pub executed_tasks: HashSet<u64>, // Executed tasks 
}