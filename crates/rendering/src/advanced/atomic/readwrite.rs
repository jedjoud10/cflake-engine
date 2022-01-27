use std::sync::{atomic::{AtomicU32, Ordering}, Arc, Mutex};

use crate::basics::transfer::{Transferable, Transfer};

// A transferable type that we can use to read back the value of a specific atomic group
#[derive(Default, Clone)]
pub struct AtomicCounterGroupRead {
    // The inner value that we will set with the atomic counters' uints
    inner: Arc<Mutex<Vec<u32>>>,
}


impl AtomicCounterGroupRead {
    // Read back the value of a single atomic using it's atomic index
    pub fn get(&self, atomic_index: usize) -> Option<u32> {
        // Get the inner value
        let lock = self.inner.lock().ok()?;
        lock.get(atomic_index).cloned()
    }
}

impl Transferable for AtomicCounterGroupRead {
    fn transfer(&self) -> Transfer<Self> {
        Transfer(Self {
            inner: self.inner.clone()
        })
    }
}