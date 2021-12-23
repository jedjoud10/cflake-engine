use others::SmartList;
use std::collections::{HashMap, HashSet};

use crate::{GPUObject, GPUObjectID};

// A simple Buffer containing the GPU objects that have been generated on the pipeline thread
#[derive(Default)]
pub struct PipelineBuffer {
    pub gpuobjects: SmartList<GPUObject>,                               // GPU objects
    pub waitable_objects: HashMap<u64, usize>,                          // Waitable ID to GPUObject index
    pub callback_objects: HashMap<u64, (usize, std::thread::ThreadId)>, // Callback ID to GPUObject index
    pub names_to_id: HashMap<String, usize>,                            // Names to GPUObject index

    // This is for tasks that have executed but that do not have a an associated GPU object
    pub executed_tasks: HashSet<u64>, // Executed tasks
}

impl PipelineBuffer {
    // Add a GPU object to the buffer, returning it's GPUObjectID
    pub fn add_gpuobject(&mut self, gpuobject: GPUObject, name: Option<String>) -> GPUObjectID {
        // Insert the gpu object
        let index = self.gpuobjects.add_element(gpuobject);
        // If we have a name add it
        match name {
            Some(name) => {
                self.names_to_id.insert(name, index);
            }
            None => {}
        }
        GPUObjectID { index: Some(index) }
    }
    // Add some additional data like callback ID or waitable ID to the GPU object
    pub fn received_new_gpu_object_additional(&mut self, id: GPUObjectID, callback_id: Option<(u64, std::thread::ThreadId)>, waitable_id: Option<u64>) {
        let index = id.index.unwrap();
        match callback_id {
            Some((id, thread_id)) => {
                self.callback_objects.insert(id, (index, thread_id));
            }
            None => { /* We cannot run a callback on this object */ }
        }
        match waitable_id {
            Some(id) => {
                self.waitable_objects.insert(id, index);
            }
            None => { /* We cannot run the un-wait function on the threads awaiting this object */ }
        }
    }
    // Get the GPUObjectID from a name
    pub fn get_id_named(&self, name: &str) -> Option<GPUObjectID> {
        let x = self.names_to_id.get(name)?;
        Some(GPUObjectID { index: Some(*x) })
    }
    // Check if a GPU object name is valid
    pub fn gpu_object_name_valid(&self, name: &str) -> bool {
        self.names_to_id.contains_key(name)
    }
    // Check if a GPU object is valid
    pub fn gpu_object_valid(&self, id: &GPUObjectID) -> bool {
        match id.index {
            Some(index) => match self.gpuobjects.get_element(index).flatten() {
                Some(_) => true,
                None => false,
            },
            None => false,
        }
    }
    // Get a GPU using it's name
    pub fn get_named_gpu_object(&self, name: &str) -> Option<GPUObject> {
        let index = self.names_to_id.get(name);
        match index {
            Some(index) => {
                let gpuobject = self.gpuobjects.get_element(*index);
                // Flatten
                gpuobject.flatten().cloned()
            }
            None => None,
        }
    }
    // Get a GPU object using it's GPUObjectID
    pub fn get_gpu_object(&self, id: &GPUObjectID) -> Option<&GPUObject> {
        self.gpuobjects.get_element(id.index?)?
    }
    // Get a GPU object using a ref usize
    pub fn get_gpu_object_usize(&self, index: &usize) -> Option<&GPUObject> {
        self.gpuobjects.get_element(*index)?
    }
    // Get multiple GPU object using a ref usize
    pub fn get_gpu_object_usize_batch<'a, I>(&self, ids: I) -> Option<Vec<&GPUObject>> 
    where
        I: Iterator<Item = &'a usize>{
        let mut gpuobjects = ids.map(|x| self.gpuobjects.get_element(*x).flatten().unwrap()).collect::<Vec<&GPUObject>>();
        Some(gpuobjects)
    }
}