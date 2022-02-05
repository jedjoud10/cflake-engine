use std::{sync::{Arc, Mutex, MutexGuard}, collections::hash_map::IterMut};
use ahash::AHashMap;
use owning_ref::MutexGuardRefMut;
use rayon::ThreadPool;

use crate::entity::EntityID;

use super::LinkedComponents;

// An enum that stores either a reference to a hashmap or an owned vector. We will use this to iterate through every LinkedComponents
pub(crate) enum ComponentQueryIterType {
    ArcHashMap(Arc<Mutex<AHashMap<EntityID, LinkedComponents>>>),
    HashMap(AHashMap<EntityID, LinkedComponents>),
}

// A struct full of LinkedComponents that we send off to update in parallel
// This will use the components data given by the world to run all the component updates in PARALLEL
// The components get mutated in parallel, though the system is NOT stored on another thread
pub struct ComponentQuery {
    // The actual components
    pub(crate) linked_components: Option<ComponentQueryIterType>,
    // The rayon thread pool available if we want to use it
    pub(crate) rayon_pool: Arc<ThreadPool>,
}

impl ComponentQuery {
    // Count the number of linked components that we have
    pub fn get_entity_count(&self) -> usize {
        let len = self.linked_components.as_ref().map(|x| match x {
            ComponentQueryIterType::ArcHashMap(x) => (x.lock().unwrap()).len(),
            ComponentQueryIterType::HashMap(x) => x.len(),
        });
        len.unwrap_or_default()
    }
    // Turn the component query into a rayon parallel iterator
    // Turn the query into a simple iterator
    pub fn into_iter<'a, F: FnOnce(IterMut<EntityID, LinkedComponents>)>(&'a mut self, function: F) -> Option<MutexGuardRefMut<'a, AHashMap<EntityID, LinkedComponents>, LinkedComponents>> {
        if self.linked_components.is_none() { return None4 };
        match self.linked_components.unwrap() {
            ComponentQueryIterType::ArcHashMap(x) => {
                let mut lock_ = x.lock().unwrap();
                let iterator = lock_.iter_mut();
                function(iterator);
            },
            ComponentQueryIterType::HashMap(mut x) => {
                let iterator = x.iter_mut();
                function(iterator);
            },
        }
    }
}
