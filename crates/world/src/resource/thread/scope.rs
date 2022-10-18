use std::sync::Arc;

use crate::{ThreadPool};
use super::ThreadedTask;

// A threadpool scope is a helper struct that allows us to send functions to execute on other threads
// A scope allows us to use immutable references to certain objects that are available in the current scope
pub struct ThreadPoolScope<'a> {
    pub(super) pool: &'a mut ThreadPool,
}

impl<'a> ThreadPoolScope<'a> {
    // Add a new task to execute in the threadpool. This task will run in the background
    // All tasks that have been sent will be completed before the current scope exits
    pub fn execute(&mut self, function: impl FnOnce() + Send + Sync + 'a) {
        type BoxFn<'b> = Box<dyn FnOnce() + Send + Sync + 'b>; 
        let function: BoxFn<'a> = Box::new(function);

        // Convert the lifetimed box into a static box
        let function: BoxFn<'static> = unsafe { 
            std::mem::transmute::<BoxFn<'a>, BoxFn<'static>>(function)
        };
        
        // Execute the task
        let task = ThreadedTask::Execute(function);
        self.pool.append(task);
    }

    // Wait till all the threads finished executing
    pub fn join(&mut self) {
        self.pool.join()
    }
}

impl<'a> Drop for ThreadPoolScope<'a> {
    fn drop(&mut self) {
        self.join();
    }
}