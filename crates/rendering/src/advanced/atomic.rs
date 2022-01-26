use std::sync::{atomic::{AtomicU32, Ordering, AtomicU8}, Arc};
use crate::basics::transfer::{Transferable, Transfer};

// The clear condition telling us when we should clear the atomic counter
#[derive(Clone)]
pub enum ClearCondition {
    BeforeShaderExecution,
    DontClear,
}

// A simple atomic counter that we can use inside OpenGL fragment and compute shaders, if possible
#[derive(Clone)]
pub struct AtomicCounter {
    pub(crate) oid: Arc<AtomicU32>,
    inner: Arc<AtomicU32>,
    pub(crate) condition: ClearCondition, 
}

impl Default for AtomicCounter {
    fn default() -> Self {
        Self { 
            oid: Arc::new(AtomicU32::new(0)),
            inner: Default::default(),
            condition: ClearCondition::DontClear
        }
    }
}

impl AtomicCounter {
    // Create a new atomic counter with a predefined value
    pub fn new(val: u32) -> Self {
        Self {
            oid: Arc::new(AtomicU32::new(0)),
            inner: Arc::new(AtomicU32::new(val)),
            condition: ClearCondition::BeforeShaderExecution
        }
    }
    // Read back the value of the atomic counter
    pub fn get(&self) -> u32 { self.inner.load(Ordering::Relaxed) }
    // Set the inner value of the atomic
    pub fn set(&self, val: u32) { self.inner.store(val, Ordering::Relaxed) }
}

impl Transferable for AtomicCounter {
    fn transfer(&self) -> Transfer<Self> {
        Transfer(self.clone())
    }
}