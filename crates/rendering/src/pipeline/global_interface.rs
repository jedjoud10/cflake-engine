use std::{collections::HashMap, sync::{atomic::AtomicBool, mpsc::Sender, Mutex}};
use crate::{RenderTaskReturn, RenderCommand, GPUObject};
use lazy_static::lazy_static;
lazy_static! {    
    // Global render interface
    pub static ref GLOBAL_INTERFACE: Mutex<GlobalInterface> = Mutex::new(GlobalInterface::default());
}

// Callback
pub struct RenderTaskCallback(Box<dyn FnOnce(RenderTaskReturn) + Send + Sync>, AtomicBool);

// Some global interface that each thread could use to send tasks / do callback shit on
#[derive(Default)]
pub struct GlobalInterface {
    // The callbacks that are for each thread
    pub callbacks: HashMap<std::thread::ThreadId, RenderTaskCallback>,
}

// Fetch the local callbacks and execute them if their corresponding task has been executed
pub fn fetch_threadlocal_callbacks() {

}

// Fetch the local callbacks and return a GPU object if we fetched one with the same name
fn fetch_threadlocal_callbacks_specific(name: &str) -> Option<GPUObject> {
    None
}

// Wait until we properly fetch a valid GPU object
pub fn wait_fetch_threadlocal_callbacks_specific(name: &str) -> GPUObject {
    let mut result = None;
    // Loop and wait until we fetch a valid one
    while result.is_none() { match fetch_threadlocal_callbacks_specific(&name) {
        Some(x) => result = Some(x),
        None => {},
    }}
    result.unwrap()
}

// We must ask the Interface if we have these objects in cache
pub fn get_gpu_object(name: &str) -> Option<GPUObject> {
    let pipeline_ = crate::PIPELINE.as_ref().lock().unwrap();
    let pipeline = pipeline_.as_ref().unwrap();
    pipeline.get_gpu_object(name).cloned()
}
pub fn gpu_object_valid(name: &str) -> bool {
    let pipeline_ = crate::PIPELINE.as_ref().lock().unwrap();
    let pipeline = pipeline_.as_ref().unwrap();
    pipeline.gpu_object_valid(name)
}