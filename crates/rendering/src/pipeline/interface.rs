use std::sync::{mpsc::Sender, RwLock};

use crate::{pipeline::buffer::GPUObjectBuffer, GPUObject, MainThreadMessage, GPUObjectID, TextureGPUObject, ShaderGPUObject, ModelGPUObject, MaterialGPUObject, SubShaderGPUObject, ComputeShaderGPUObject, TextureFillGPUObject, RendererGPUObject};
use lazy_static::lazy_static;

lazy_static! {
    static ref INTERFACE_BUFFER: RwLock<GPUObjectBuffer> = RwLock::new(GPUObjectBuffer::default());
}

/* #region Get GPU objects using their GPUObjectID or their name */
// Get GPU object using it's specified name
pub fn get_named_gpu_object(name: &str) -> Option<GPUObject> {
    let buf = INTERFACE_BUFFER.read().unwrap();
    let index = buf.names_to_id.get(name);
    match index {
        Some(index) => {
            let gpuobject = buf.gpuobjects.get_element(*index);
            // Flatten
            gpuobject.flatten().cloned()
        }
        None => None,
    }
}
// Get a GPU object using it's GPUObjectID
pub fn get_gpu_object<'a>(id: &'a GPUObjectID) -> Option<&GPUObject> {
    let buf = INTERFACE_BUFFER.read().unwrap();
    let gpuobject = buf.gpuobjects.get_element(id.index?);
    // Flatten
    let x = gpuobject.flatten()?;
    let ptr = x as *const GPUObject;
    // Gonna document later why this is totally safe and how it *should* not cause a mem corruption or UB
    Some(unsafe { &*ptr })
}
// Get a GPU object using a ref usize
pub fn get_gpu_object_usize<'a>(id: &'a usize) -> Option<&GPUObject> {
    let buf = INTERFACE_BUFFER.read().unwrap();
    let gpuobject = buf.gpuobjects.get_element(*id);
    // Flatten
    let x = gpuobject.flatten()?;
    let ptr = x as *const GPUObject;
    // Gonna document later why this is totally safe and how it *should* not cause a mem corruption or UB
    Some(unsafe { &*ptr })
}
// Get the GPUObjectID from a name
pub fn get_id_named(name: &str) -> Option<GPUObjectID> {
    let buf = INTERFACE_BUFFER.read().unwrap();
    let x = buf.names_to_id.get(name)?;
    Some(GPUObjectID { index: Some(*x) })
}
// Check if a GPU object name is valid
pub fn gpu_object_name_valid(name: &str) -> bool {
    let buf = INTERFACE_BUFFER.read().unwrap();
    buf.names_to_id.contains_key(name)
}
// Check if a GPU object is valid
pub fn gpu_object_valid(id: &GPUObjectID) -> bool {
    let buf = INTERFACE_BUFFER.read().unwrap();
    match id.index {
        Some(index) => match buf.gpuobjects.get_element(index).flatten() {
            Some(_) => true,
            None => false,
        },
        None => false,
    }   
}
/* #endregion */


// Notify the threads that we have recieved a valid GPU object
pub fn received_new_gpu_object(gpuobject: GPUObject, name: Option<String>) -> GPUObjectID {
    // Add the GPU object to the current interface buffer
    let mut buf = INTERFACE_BUFFER.write().unwrap();
    // Always insert the gpu object
    let index = buf.gpuobjects.add_element(gpuobject);
    // If we have a name add it
    match name {
        Some(name) => {
            buf.names_to_id.insert(name, index);
        }
        None => {}
    }
    GPUObjectID { index: Some(index) }
}

// Additional data given to the interface after we add a GPU object
pub fn received_new_gpu_object_additional(gpuobject_id: GPUObjectID, callback_id: Option<(u64, std::thread::ThreadId)>, waitable_id: Option<u64>) {   
    let mut buf = INTERFACE_BUFFER.write().unwrap(); 
    let index = gpuobject_id.index.unwrap();
    match callback_id {
        Some((id, thread_id)) => {
            buf.callback_objects.insert(id, (index, thread_id));
        }
        None => { /* We cannot run a callback on this object */ }
    }
    match waitable_id {
        Some(id) => {
            buf.waitable_objects.insert(id, index);
        }
        None => { /* We cannot run the un-wait function on the threads awaiting this object */ }
    }
}

// Remove a GPU object from the interface buffer
pub fn remove_gpu_object(gpuobject_id: GPUObjectID) {
    let mut buf = INTERFACE_BUFFER.write().unwrap();
    buf.gpuobjects.remove_element(gpuobject_id.index.unwrap());
}

// Update a GPU object using a callback
pub fn update_gpu_object<F>(gpuobject_id: GPUObjectID, f: F) where F: FnOnce(&mut GPUObject) {
    let mut buf = INTERFACE_BUFFER.write().unwrap();
    let x = buf.gpuobjects.get_element_mut(gpuobject_id.index.unwrap()).flatten().unwrap();
    f(x);
}

// We have received confirmation that we have executed a specific task
pub fn received_task_execution_ack(execution_id: u64) {
    // Add the GPU object to the current interface buffer
    let mut buf = INTERFACE_BUFFER.write().unwrap();
    buf.executed_tasks.insert(execution_id);
}

// Update the render thread, and call the callbacks of GPU objects that have been created
pub fn update_render_thread(tx2: &Sender<MainThreadMessage>) {
    // Send a message to the main thread saying what callbacks we must run
    let mut buf = INTERFACE_BUFFER.write().unwrap();
    let callbacks_objects_indices = std::mem::take(&mut buf.callback_objects);
    let callback_objects = callbacks_objects_indices
        .into_iter()
        .map(|(callback_id, (index, thread_id))| 
            (callback_id,
            (buf.gpuobjects.get_element(index).unwrap().cloned().unwrap(),
            GPUObjectID { index: Some(index) },),
            thread_id))
        .collect::<Vec<(u64, (GPUObject, GPUObjectID), std::thread::ThreadId)>>();

    // Now we must all of this to the main thread
    for (callback_id, args, thread_id) in callback_objects {
        tx2.send(MainThreadMessage::ExecuteCallback(callback_id, args, thread_id)).unwrap();
    }
}

// Wait for the result of a specific GPU object, specified with it's special waitable ID
pub fn wait_for_gpuobject_id(id: u64) -> Option<GPUObjectID> {
    // Basically an infinite loop waiting until we poll a valid GPU object using the specified ID
    loop {
        let buf = INTERFACE_BUFFER.read().unwrap();
        match buf.waitable_objects.get(&id) {
            Some(&gpuobject_index) => {
                return Some(GPUObjectID { index: Some(gpuobject_index) });
            }
            None => {}
        }

        // Check if we have quit the render loop, because if we did, this will never exit and we must manually exit
        let barrier_data = others::barrier::as_ref();
        if barrier_data.is_world_destroyed() {
            return None;
        }
    }
}

// Wait for the execution of a specific task
pub fn wait_for_execution(id: u64) {
    // Basically an infinite loop waiting until we poll a valid GPU object using the specified ID
    loop {
        let buf = INTERFACE_BUFFER.read().unwrap();
        if buf.executed_tasks.contains(&id) {
            // We have executed this task, we can exit
            return;
        }

        // Check if we have quit the render loop, because if we did, this will never exit and we must manually exit
        let barrier_data = others::barrier::as_ref();
        if barrier_data.is_world_destroyed() {
            return;
        }
    }
}
