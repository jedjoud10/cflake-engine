use std::{alloc::Layout, ffi::c_void};

use crate::{registry, Component, Mask};

// Component storage pointer
#[derive(Clone)]
pub(crate) struct StorageVecPtr {
    // The underlying pointer to the vector. This will point to garbage if the vector gets reallocated
    pub ptr: *mut c_void,
}

impl StorageVecPtr {
    // Very unsafe, but in our case totally fine since we read this only from queries or entries
    pub unsafe fn get_ptr_unchecked<T: Component>(&self, idx: usize) -> *mut T {
        (self.ptr as *mut T).add(idx)
    }
}
