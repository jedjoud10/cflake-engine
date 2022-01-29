use std::{sync::{RwLock, Arc, atomic::{AtomicUsize, Ordering}}, cell::{UnsafeCell, Cell}, iter};

// The power of atomics
thread_local! {
    pub(crate) static CURRENT_EXECUTION_INDEX: Cell<usize> = Cell::new(0);
}

// Creates a vector that we can modify inside the worker threads
// This is totally safe, since we will not be accessing multiple elements while they are being written to
pub struct SharedVec<T> {
    // The underlying vector
    pub(crate) vec: Arc<Vec<UnsafeCell<T>>>,
}

unsafe impl<T> Sync for SharedVec<T> {}

impl<T> SharedVec<T> {
    // Create a new shared vector using a specific len
    pub fn new(len: usize) -> Self
    where T:
        Default
    {
        let mut vec = Vec::with_capacity(len);
        for x in 0..len { vec.push(UnsafeCell::default()); }
        unsafe { vec.set_len(len); }
        Self {
            vec: Arc::new(vec)
        }
    }
    // Read the current element
    pub fn read(&self) -> Option<&T> {

        // Then get the index and the unsafe cell
        let idx = CURRENT_EXECUTION_INDEX.with(|x| x.get());
        let cell = self.vec.get(idx)?;

        // Then we can read from the cell
        Some(unsafe { &*cell.get() })
    }
    // Write the current element
    pub fn write(&self) -> Option<&mut T> {
        // Then get the index and the unsafe cell
        let idx = CURRENT_EXECUTION_INDEX.with(|x| x.get());
        let cell = self.vec.get(idx)?;

        // Then we can read from the cell
        Some(unsafe { &mut *cell.get() })
    }
}