use super::{query_guard::ComponentQueryGuard, LinkedComponents};
use crate::entity::EntityID;
use ahash::AHashMap;
use rayon::ThreadPool;
use std::sync::{Arc, Mutex};

// A struct full of LinkedComponents that we send off to update in parallel
// This will use the components data given by the world to run all the component updates in PARALLEL
// The components get mutated in parallel, though the system is NOT stored on another thread
pub struct ComponentQuery {
    // The actual components
    pub(crate) linked_components: Option<Arc<Mutex<AHashMap<EntityID, LinkedComponents>>>>,
    // The rayon thread pool available if we want to use it
    pub(crate) rayon_pool: Arc<ThreadPool>,
}

impl ComponentQuery {
    // Count the number of linked components that we have
    pub fn get_entity_count(&self) -> usize {
        let len = self.linked_components.as_ref().map(|x| x.lock().unwrap().len());
        len.unwrap_or_default()
    }
    // Lock the component query, returning a ComponentQueryGuard that we can use to iterate over the components
    pub fn lock<'a>(&'a mut self) -> ComponentQueryGuard<'a> {
        let locked = self.linked_components.as_ref().unwrap().lock().unwrap();
        ComponentQueryGuard { inner: locked }
    }
    // Get the rayon thread pool
    pub fn get_thread_pool(&self) -> &ThreadPool {
        self.rayon_pool.as_ref()
    }
}
