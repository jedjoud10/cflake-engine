use std::sync::{atomic::AtomicU32, Arc};

use crate::basics::transfer::{Transferable, Transfer};

// A simple atomic counter that we can use inside OpenGL fragment and compute shaders, if possible
#[derive(Default, Clone)]
pub struct AtomicCounter {
    // The OpenGL atomic counter buffer name
    oid: u32,
    // The atomic counter's inner value
    inner: Arc<AtomicU32>, 
}

impl AtomicCounter {
    // Create a new atomic counter with a predefined value
    pub fn new(val: u32) -> Self {
        Self {
            inner: Arc::new(AtomicU32::new(val)),
            ..Self::default()
        }
    }
}

impl Transferable for AtomicCounter {
    fn transfer(&self) -> Transfer<Self> {
        Transfer(self.clone())
    }
}