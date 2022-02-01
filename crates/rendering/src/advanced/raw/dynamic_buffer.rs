use std::{ptr::null, mem::size_of, ffi::c_void};

use crate::utils::UsageType;

// A dynamic OpenGL buffer that automatically reallocates it's size when we add to many elements to it
pub struct DynamicRawBuffer<T> {
    // The OpenGL data for this buffer
    pub oid: u32,
    _type: u32,
    
    // Other data
    usage: UsageType,
    vec: Vec<T>
}

impl<T> DynamicRawBuffer<T> {
    // Create the dynamic raw buffer
    // This can only be called on the render thread
    pub unsafe fn new(_type: u32, usage: UsageType) -> Self {
        Self::with_capacity(_type, 0, usage)
    }
    // Create a new dynamic raw buffer with a specified capacity
    pub unsafe fn with_capacity(_type: u32, capacity: usize, usage: UsageType) -> Self {
        let vec = Vec::<T>::with_capacity(capacity);
        let oid = {
            let mut oid = 0;
            gl::GenBuffers(1, &mut oid);
            gl::BindBuffer(_type, oid);
            gl::BufferData(_type, 0, null(), usage.convert());
            oid
        };
        Self {
            oid,
            _type,
            vec,
            usage,
        }
    }
    // Add an element to the raw buffer
    // This may reallocate the OpenGL buffer if it's last len is insufficient
    pub unsafe fn push(&mut self, val: T) {
        // Get our old capacity and compare with our new capacity
        let old_capacity = self.vec.capacity();
        // Add the element to our rust vector anyways
        self.vec.push(val);

        // Reallocate the OpenGL buffer if needed
        gl::BindBuffer(self._type, self.oid);
        if self.vec.capacity() > old_capacity {
            // Reallocate
            gl::BufferData(self._type, (size_of::<T>() * self.vec.len()) as isize, self.vec.as_ptr() as *const c_void, self.usage.convert());
        } else {
            // We don't need to reallocate, we just need to update our sub-data
            let offset = (self.vec.len()-1) * size_of::<T>();
            let data = self.vec.last().unwrap();
            gl::BufferSubData(self._type, offset as isize, size_of::<T>() as isize, data as *const T as *const c_void);
        }
    }
}