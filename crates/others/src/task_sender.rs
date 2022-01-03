use std::{marker::PhantomData, sync::Arc};

use crossbeam::queue::SegQueue;

// A task sender that will be cloned on each thread that we will have
// This will allow us to send World Tasks or Pipeline Tasks from System Component Update Threads
#[derive(Clone)]
pub struct TaskSender<T> {
    queue: Arc<SegQueue<T>>,
}

impl<T> TaskSender<T> {
    // Send a specific task to the target thread
    pub fn send(&self, task: T) {
        self.queue.as_ref().push(task);
    }
    // Read the tasks on the target thread
    pub fn clear_tasks(&mut self) -> Vec<T> {
        let mut tasks: Vec<T> = Vec::new();
        loop {
            let elem = self.queue.as_ref().pop();
            if elem.is_none() { return tasks; /* The queue is empty */ }
            tasks.push(elem.unwrap());
        }
    }
}
