use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::{pipeline::buffer::GlobalBuffer, GPUObjectID};
use super::buffer::GLOBAL_BUFFER;

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