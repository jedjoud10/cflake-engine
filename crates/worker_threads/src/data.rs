use std::ptr::{null_mut, null};

// The data that will be stored in an atomic pointer
pub struct SharedData<C, T: Sync> {
    // Elements, and the fn pointer
    pub elements: Vec<*mut T>,
    pub function: fn(&C, usize, &mut T),
    // A context (World as example)
    pub context: *const C,
    // Some chunk distribution data
    pub chunk_size: usize,
}

unsafe impl<C, T: Sync> Send for SharedData<C, T> {}
unsafe impl<C, T: Sync> Sync for SharedData<C, T> {}

impl<C, T: Sync> Default for SharedData<C, T> {
    fn default() -> Self {
        Self {
            elements: Vec::new(),
            function: |_, _, _| {},
            context: null(),
            chunk_size: 32,
        }
    }
}