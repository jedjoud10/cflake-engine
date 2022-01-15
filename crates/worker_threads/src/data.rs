use std::ptr::{null, null_mut};

// The data that will be stored in an atomic pointer
pub struct SharedData<T> {
    // Elements, and the fn pointer
    pub elements: Vec<*mut T>,
    pub function: Box<dyn Fn(&mut T)>,
    // Some chunk distribution data
    pub chunk_size: usize,
}

unsafe impl<T> Send for SharedData<T> {}
unsafe impl<T> Sync for SharedData<T> {}

impl<T> Default for SharedData<T> {
    fn default() -> Self {
        Self {
            elements: Vec::new(),
            function: Box::new(|_| {}),
            chunk_size: 32,
        }
    }
}
