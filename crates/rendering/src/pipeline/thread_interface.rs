use crate::GPUObject;


// Update the render thread, and call the callbacks of GPU objects that have been created
pub fn update_render_thread() {
}

// Update the current thread, checking if we must run any callbacks or not
pub fn update_thread_worker() {
}

// Wait for the result of a specific GPU object, specified with it's special waitable ID
pub fn wait_for_gpuobject(id: u64) -> GPUObject {
    
}