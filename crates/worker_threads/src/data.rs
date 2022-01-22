

// The data that will be stored in an atomic pointer
pub struct SharedData<T> {
    // Elements, and the fn pointer
    pub elements: Vec<*mut T>,
    pub function: Option<*const dyn Fn(&mut T)>,
    // Some chunk distribution data
    pub chunk_size: usize,
}

impl<T> SharedData<T> {
    pub fn clear(&mut self) {
        self.elements.clear();
        self.function.take();
        self.chunk_size = 0;
    }
}

unsafe impl<T> Send for SharedData<T> {}
unsafe impl<T> Sync for SharedData<T> {}

impl<T> Default for SharedData<T> {
    fn default() -> Self {
        Self {
            elements: Vec::new(),
            function: None,
            chunk_size: 32,
        }
    }
}
