use crate::{GPUObject, pipeline::buffer::GPUObjectBuffer};
use lazy_static::lazy_static;
use no_deadlocks::{RwLock, Mutex};

lazy_static! {
    static ref INTERFACE_BUFFER: Mutex<GPUObjectBuffer> = Mutex::new(GPUObjectBuffer::default());
}

/* #region Get GPU objects using their waitable ID or their name */
// Get GPU object using it's specified name
pub fn get_gpu_object(name: &str) -> Option<GPUObject> {
    let buf = INTERFACE_BUFFER.lock().unwrap();
    let index = buf.names_to_id.get(name);
    match index {
        Some(index) => {
            let gpuobject = buf.gpuobject.get_element(*index);
            // Flatten
            gpuobject.flatten().cloned()
        },
        None => None,
    }
    
}
// Check if a GPU object exists
pub fn gpu_object_valid(name: &str) -> bool {
    let buf = INTERFACE_BUFFER.lock().unwrap();
    buf.names_to_id.contains_key(name)
}
/* #endregion */

// Notify the threads that we have recieved a valid GPU object
pub fn received_new_gpu_object(gpuobject: GPUObject, callback_id: Option<u64>, waitable_id: Option<u64>) {
    // Add the GPU object to the current interface buffer
    let mut buf = INTERFACE_BUFFER.lock().unwrap();
    // Always insert the gpu object
    let index = buf.gpuobject.add_element(gpuobject);
    match callback_id {
        Some(id) => { buf.callback_objects.insert(id, index); },
        None => { /* We cannot run a callback on this object */ },
    }
    match waitable_id {
        Some(id) => { buf.waitable_objects.insert(id, index); },
        None => { /* We cannot run the un-wait function on the threads awaiting this object */ },
    }    
}

// We have received confirmation that we have executed a specific task
pub fn received_task_execution_ack(execution_id: u64) {
    // Add the GPU object to the current interface buffer
    let mut buf = INTERFACE_BUFFER.lock().unwrap();
    buf.executed_tasks.insert(execution_id);
}

// Update the render thread, and call the callbacks of GPU objects that have been created
pub fn update_render_thread() {
}

// Update the current thread, checking if we must run any callbacks or not
pub fn update_thread_worker() {
}

// Wait for the result of a specific GPU object, specified with it's special waitable ID
pub fn wait_for_gpuobject(id: u64) -> GPUObject {
    // Basically an infinite loop waiting until we poll a valid GPU object using the specified ID
    loop {
        let buf = INTERFACE_BUFFER.lock().unwrap();
        match buf.callback_objects.get(&id) {
            Some(&gpuobject_index) => {
                let gpuobject =  buf.gpuobject.get_element(gpuobject_index).unwrap().unwrap();
                // Was able to poll a valid GPU object
                return gpuobject.clone();
            },
            None => {},
        }        
    }
}

// Wait for the execution of a specific task
pub fn wait_for_execution(id: u64) {
    // Basically an infinite loop waiting until we poll a valid GPU object using the specified ID
    loop {
        let buf = INTERFACE_BUFFER.lock().unwrap();
        if buf.executed_tasks.contains(&id) {
            // We have executed this task, we can exit
            return;
        }        
    }
}