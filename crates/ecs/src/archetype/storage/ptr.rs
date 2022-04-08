use std::{alloc::Layout, ffi::c_void, sync::{atomic::{AtomicPtr, Ordering}, Arc}};

use crate::{registry, Component, Mask};

// Component storage pointer
#[derive(Clone)]
pub(crate) struct StorageVecPtr {
    // The underlying pointer to the vector. This will point to garbage if the vector gets reallocated
    ptr: *mut c_void,
}

impl StorageVecPtr {
    // Create a new storage vec ptr from a vector
    pub fn new<T: Component>(vec: &mut Vec<T>) -> Self {
        Self {
            ptr: vec.as_mut_ptr() as *mut c_void,
        }
    }
    // Get the underlying storage pointer
    pub fn get_ptr<T: Component>(&self) -> *mut T {
        self.ptr as *mut T
    }
    // Very unsafe, but in our case totally fine since we read this only from queries or entries
    pub unsafe fn get_bundle_ptr_unchecked<T: Component>(&self, idx: usize) -> *mut T {
        self.get_ptr::<T>().add(idx)
    }
}
