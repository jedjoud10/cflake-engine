use std::sync::{Arc, Mutex};

use worker_threads::ThreadPool;

use super::{LinkedComponents, StoredGlobalComponents};

// A struct full of LinkedComponents that we send off to update in parallel
// This will use the components data given by the world to run all the component updates in PARALLEL
// The components get mutated in parallel, though the system is NOT stored on another thread
pub struct ComponentQuery {
    // The actual components
    pub(crate) linked_components: Option<Vec<LinkedComponents>>,
    // We will also pass it all the global components that we need for this system
    pub(crate) stored_global_components: Option<StoredGlobalComponents>,
    // Thread pool because I am insane
    pub(crate) thread_pool: Arc<Mutex<ThreadPool<LinkedComponents>>>,
}

impl ComponentQuery {
    // Get the stored global component so we can access them
    pub fn get_global_components(&mut self) -> Option<&mut StoredGlobalComponents> {
        None
    }
    // Update all the components consecutively, on the main thread
    pub fn update_all<F: Fn(&mut LinkedComponents) + 'static>(self, function: F) {
        // Run it normally
        if let Some(vec) = self.linked_components {
            for mut linked_components in vec {
                function(&mut linked_components);
            }
        }
    }
    // Update all the components consecutively, on the main thread, but while also mapping each element and returning a new vector
    pub fn update_all_map<U, F: Fn(&mut LinkedComponents) -> Option<U> + 'static>(self, function: F) -> Vec<U> {
        // Make a new vector the size of self.linked_components
        let mut output_vec = Vec::with_capacity(self.linked_components.as_ref().map(|x| x.len()).unwrap_or_default());
        if let Some(vec) = self.linked_components {
            for mut linked_components in vec {
                let output = function(&mut linked_components);
                if let Some(output) = output { output_vec.push(output); }
            }
        }
        output_vec
    }
    // Update all the components in parallel, on multiple worker threads
    pub fn update_all_threaded<F: Fn(&mut LinkedComponents) + 'static + Sync + Send>(self, function: F) {
        if let Some(mut vec) = self.linked_components {
            let thread_pool = self.thread_pool.lock().unwrap();
            thread_pool.execute(&mut vec, function);
        }
    }
}
