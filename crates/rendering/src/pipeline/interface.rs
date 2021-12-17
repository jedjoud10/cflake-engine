use crate::GPUObject;
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        RwLock, RwLockReadGuard, RwLockWriteGuard,
    },
};
lazy_static! {
    // Global render interface
    static ref GLOBAL_INTERFACE: RwLock<GlobalInterface> = RwLock::new(GlobalInterface::default());
}

// Get the interface immutably
fn interface() -> RwLockReadGuard<'static, GlobalInterface> {
    GLOBAL_INTERFACE.read().unwrap()
}

fn interface_mut() -> RwLockWriteGuard<'static, GlobalInterface> {
    GLOBAL_INTERFACE.write().unwrap()
}

struct CallbackData(pub Box<dyn Fn(GPUObject) + Send + Sync + 'static>, pub GPUObject, pub AtomicBool, pub bool);

// Some global interface that each thread could use to send tasks / do callback shit on
#[derive(Default)]
struct GlobalInterface {
    // The callbacks that we have received from the render thread
    pub callbacks: HashMap<std::thread::ThreadId, HashMap<String, CallbackData>>,
}

// We receive a valid GPU object from the pipeline
pub fn executed_task(thread_id: std::thread::ThreadId, name: String, gpuobject: GPUObject, callback: Box<dyn Fn(GPUObject) + Send + Sync + 'static>, waitable: bool) {
    let mut i = interface_mut();
    let new_callback = CallbackData(callback, gpuobject, AtomicBool::new(false), waitable);
    let entry = i.callbacks.entry(thread_id).or_default();
    entry.insert(name, new_callback);
}

// Fetch the local callbacks and execute them if their corresponding task has been executed
pub fn fetch_threadlocal_callbacks() {
    let interface = interface();
    let thread_id = std::thread::current().id();
    // Call all the callbacks in this thread if possible
    match interface.callbacks.get(&thread_id) {
        Some(callbacks) => {
            for (_name, callback_data) in callbacks.iter() {
                // Call all the callbacks in this worker thread
                if !callback_data.2.load(Ordering::Relaxed) && !callback_data.3 {
                    // This callback was not executed yet
                    let callback = &callback_data.0;
                    let gpuobject = callback_data.1.clone();
                    (callback)(gpuobject.clone());
                    // Update the atomic bool
                    callback_data.2.fetch_or(true, Ordering::Relaxed);
                } else { /* Already called, don't call it again */
                }
            }
        }
        None => { /* No callbacks for this thread yet */ }
    }
}

// The update loop (end of the frame) on the rendering thread
pub fn update_render_thread() {
    let mut interface = interface_mut();
    // Delete all the callbacks that have been used
    for (_thread_id, callbacks) in interface.callbacks.iter_mut() {
        callbacks.retain(|_key, callback_data| !callback_data.2.load(Ordering::Relaxed));
    }
}

// Fetch the local callbacks and return a GPU object if we fetched one with the same name
fn fetch_threadlocal_callbacks_specific(name: &str) -> Option<GPUObject> {
    let interface = interface();
    let thread_id = std::thread::current().id();
    match interface.callbacks.get(&thread_id) {
        Some(callbacks) => {
            match callbacks.get(name) {
                Some(callback_data) => {
                    // Call the callback if possible
                    if callback_data.3 {
                        // This callback was not executed yet
                        let _callback = &callback_data.0;
                        let gpuobject = callback_data.1.clone();
                        // Update the atomic bool
                        callback_data.2.fetch_or(true, Ordering::Relaxed);
                        Some(gpuobject)
                    } else {
                        /* This callback is not waitable */
                        None
                    }
                }
                None => {
                    /* Callback with that specific name does not exist */
                    None
                }
            }
        }
        None => {
            /* No callbacks for this thread yet */
            None
        }
    }
}

// Wait until we properly fetch a valid GPU object
pub fn wait_fetch_threadlocal_callbacks_specific(name: &str) -> GPUObject {
    let mut result = None;
    // Loop and wait until we fetch a valid one
    while result.is_none() {
        match fetch_threadlocal_callbacks_specific(name) {
            Some(x) => result = Some(x),
            None => { /* We wait */ }
        }
    }
    result.unwrap()
}

// We must ask the Interface if we have these objects in cache
pub fn get_gpu_object(name: &str) -> Option<GPUObject> {
    let pipeline_ = crate::pipeline();
    let pipeline = pipeline_.as_ref().unwrap();
    pipeline.get_gpu_object(name).cloned()
}
pub fn gpu_object_valid(name: &str) -> bool {
    let pipeline_ = crate::pipeline();
    let pipeline = pipeline_.as_ref().unwrap();
    pipeline.gpu_object_valid(name)
}
