use lazy_static::lazy_static;
use std::{borrow::BorrowMut, cell::RefCell, collections::HashMap, sync::atomic::AtomicU64};

lazy_static! {
    static ref CALLBACK_COUNTER: AtomicU64 = AtomicU64::new(0); // The number of callbacks that have been created
}

// Execute a specific callback on this thread
pub fn execute_callback<T: Callback, U: CallbackArguments>(id: u64, callback_arguments: U, callback_manager: &mut CallbackManagerBuffer<T>) {
    let callback = callback_manager.callbacks.remove(&id).unwrap();
    callback.call(callback_arguments);
}

pub trait CallbackArguments {}

// Create a callback ID from self
pub trait IDCallbackCreate where Self: Sized {
    // Create the callback and get back it's ID
    fn create(self) -> u64 {
        let id = CALLBACK_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        id
        // Here we must add the callback to the thread local callback manager
    }
}

// A main callback trait that can be implemented for callbacks stored on the buffer
pub trait Callback where Self: Sized {
    // Call this callback
    fn call<U>(self, callback_arguments: U);
}

// The main callback manager that is stored on the main thread, and that sends commands to the system threads that must execute their callbacks
// Callback manager that contains all the current callbacks (Thread Local)
#[derive(Default)]
pub struct CallbackManagerBuffer<T> where T: Callback {
    callbacks: HashMap<u64, T>,
}

impl<T> CallbackManagerBuffer<T> where T: Callback {
    // Add a callback to this thread local buffer
    pub fn add_callback(&mut self, id: u64, callback: T) {
        self.callbacks.insert(id, callback);
    }
}



// A ref callback, always ran at the end of the current system frame
pub struct RefCallback<T> {
    pub callback: Box<dyn Fn(&T)>,
}
// A mutable callback that mutates that value passed. Always ran at the end of the world thread frame
pub struct MutCallback<T> {
    pub callback: Box<dyn Fn(&mut T)>,
}

impl<T> RefCallback<T> {
    pub fn new<F>(c: F) -> Self
    where
        F: Fn(&T) + 'static,
    {
        let callback = Box::new(c);
        Self { callback }
    }
}

impl<T> MutCallback<T> {
    pub fn new<F>(c: F) -> Self
    where
        F: Fn(&mut T) + 'static,
    {
        let callback = Box::new(c);
        Self { callback }
    }
}
