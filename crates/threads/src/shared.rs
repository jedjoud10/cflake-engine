use std::{sync::{RwLock, Arc, atomic::{AtomicUsize, Ordering}}, cell::{UnsafeCell, Cell}, iter};
use crate::IterExecutionID;

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
    pub fn read(&self, id: &IterExecutionID) -> Option<&T> {
        // Then get the index and the unsafe cell
        let cell = self.vec.get(id.info.element_index)?;

        // Then we can read from the cell
        Some(unsafe { &*cell.get() })
    }
    // Write the current element
    pub fn write(&self, id: &IterExecutionID) -> Option<&mut T> {
        // Then get the index and the unsafe cell
        let cell = self.vec.get(id.info.element_index)?;

        // Then we can read from the cell
        Some(unsafe { &mut *cell.get() })
    }
    // Turn this shared vec into it's safe counterpart
    pub fn into_inner(self) -> Vec<T> {
        let vec = Arc::try_unwrap(self.vec).unwrap();
        vec.into_iter().map(|x| x.into_inner()).collect::<Vec<_>>()
    }
}