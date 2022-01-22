use std::sync::{Arc, Mutex};

use worker_threads::ThreadPool;

use super::LinkedComponents;

// A struct full of LinkedComponents that we send off to update in parallel
// This will use the components data given by the world to run all the component updates in PARALLEL
// The components get mutated in parallel, though the system is NOT stored on another thread
pub struct ComponentQuery {
    // The actual components
    pub(crate) linked_components: Option<Vec<LinkedComponents>>,
    // Thread pool because I am insane
    pub(crate) thread_pool: Arc<Mutex<ThreadPool<LinkedComponents>>>,
}

impl ComponentQuery {
    // Update all the components consecutively, on the main thread
    pub fn update_all<F: Fn(&mut LinkedComponents) + 'static>(self, function: F) {
        // Run it normally
        if let Some(vec) = self.linked_components {
            for mut linked_components in vec {
                function(&mut linked_components);
            }
        }
    }
    // Update all the components in parallel, on multiple worker threads
    pub fn update_all_threaded<F: Fn(&mut LinkedComponents) + 'static + Sync + Send>(self, function: F) {
        if let Some(mut vec) = self.linked_components {
            let thread_pool = self.thread_pool.lock().unwrap();
            thread_pool.execute(&mut vec, function);
        }
    }
}
