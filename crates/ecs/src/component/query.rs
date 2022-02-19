use super::{query_guard::{MutComponentQuery, RefComponentQuery}, LinkedComponents};
use crate::entity::EntityID;
use ahash::AHashMap;
use rayon::ThreadPool;
use std::{cell::RefCell, rc::Rc};

// A struct full of LinkedComponents that we send off to update in parallel
// This will use the components data given by the world to run all the component updates in PARALLEL
// The components get mutated in parallel, though the system is NOT stored on another thread
pub struct ComponentQuery {
    // The actual components
    pub(crate) linked_components: Option<Rc<RefCell<AHashMap<EntityID, LinkedComponents>>>>,
    // The rayon thread pool available if we want to use it
    pub(crate) rayon_pool: Rc<ThreadPool>,
}

impl ComponentQuery {
    // Count the number of linked components that we have
    pub fn get_entity_count(&self) -> usize {
        let len = self.linked_components.as_ref().map(|x| x.borrow().len());
        len.unwrap_or_default()
    }
    // Lock the component query, returning a ComponentQueryGuard that we can use to iterate over the components
    pub fn write(&self) -> MutComponentQuery {
        let locked = self.linked_components.as_ref().unwrap().borrow_mut();
        MutComponentQuery { inner: locked }
    }
    pub fn read(&self) -> RefComponentQuery {
        let locked = self.linked_components.as_ref().unwrap().borrow();
        RefComponentQuery { inner: locked }
    }
    // Get the rayon thread pool
    pub fn get_thread_pool(&self) -> &ThreadPool {
        self.rayon_pool.as_ref()
    }
}
