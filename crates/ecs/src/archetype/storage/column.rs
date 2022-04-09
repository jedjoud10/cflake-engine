use std::{
    any::Any,
    ffi::c_void,
    ptr::{null, null_mut},
};

use smallvec::SmallVec;

use crate::{Mask, StorageVec, StorageVecPtr};

// A storage column, it contains a pointer along side the boxed storage
pub(crate) struct StorageColumn {
    // Boxed vector storage
    boxed: Box<dyn StorageVec>,

    // Our raw pointer (this is null)
    ptr: *mut c_void,
}

impl StorageColumn {
    // Create a new storage column from a boxed storage vec
    pub fn new(mask: Mask, boxed: Box<dyn StorageVec>) -> Self {
        let ptr = boxed.get_null_mut_typeless_ptr();
        Self { boxed, ptr }
    }
    // Push a new boxed element into the vector, and update the internally stored ptr in case we reallocate
    pub fn push(&mut self, component: Box<dyn Any>) {
        self.boxed.push(component);
        self.ptr = self.boxed.as_mut_typeless_ptr();
    }
    // Swap remove a specific component
    pub fn swap_remove_bundle(&mut self, bundle: usize) {
        self.boxed.swap_remove_bundle(bundle)
    }
    // Swap remove a specific component, but boxes the result so we can return it
    pub fn swap_remove_boxed_bundle(&mut self, bundle: usize) -> Box<dyn Any> {
        self.boxed.swap_remove_boxed_bundle(bundle)
    }
    // Get the cached pointer, since we only have a &self context
    pub fn get_ptr(&self) -> *mut c_void {
        self.ptr
    }
}
