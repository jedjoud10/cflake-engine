use std::{any::Any, ptr::null_mut};

use crate::{Mask, StorageVec, StorageVecPtr};

// A storage column, it contains a pointer along side the boxed storage
pub(crate) struct StorageColumn {
    // Boxed vector storage
    pub boxed: Box<dyn StorageVec>,

    // It's raw ptr. This will change everytime the vector reallocates
    pub ptr: Option<StorageVecPtr>,
}

impl StorageColumn {
    // Create a new storage column from a boxed storage vec
    pub fn new(mask: Mask, boxed: Box<dyn StorageVec>) -> Self {
        Self { boxed, ptr: None }
    }
    // Push a new boxed element into the vector, and update the internally stored ptr in case we reallocate
    pub fn push(&mut self, component: Box<dyn Any>) {
        self.boxed.push(component);
        self.ptr.insert(self.boxed.as_storage_ptr().unwrap());
    }
    // Swap remove a specific component
    pub fn swap_remove_bundle(&mut self, bundle: usize) {
        self.boxed.swap_remove_bundle(bundle)
    }
    // Swap remove a specific component, but boxes the result so we can return it
    pub fn swap_remove_boxed_bundle(&mut self, bundle: usize) -> Box<dyn Any> {
        self.boxed.swap_remove_boxed_bundle(bundle)
    }
}
