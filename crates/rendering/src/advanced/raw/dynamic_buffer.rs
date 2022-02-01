use std::{ptr::null, mem::size_of, ffi::c_void};

use crate::utils::UsageType;

// A dynamic OpenGL buffer that automatically reallocates it's size when we add to many elements to it
pub struct DynamicRawBuffer<T> {
    // The OpenGL data for this buffer
    pub buffer: u32,
    _type: u32,
    
    // Other data
    usage: UsageType,
    vec: Vec<T>
}

impl<T> DynamicRawBuffer<T> {
    // Create the dynamic raw buffer
    // This can only be called on the render thread
    pub fn new(_type: u32, usage: UsageType) -> Self {
        Self::with_capacity(_type, 0, usage)
    }
    // Create a new dynamic raw buffer with a specified capacity
    pub fn with_capacity(_type: u32, capacity: usize, usage: UsageType) -> Self {
        let vec = Vec::<T>::with_capacity(capacity);
        let oid = unsafe {
            let mut oid = 0;
            gl::GenBuffers(1, &mut oid);
            gl::BindBuffer(_type, oid);
            gl::BufferData(_type, (size_of::<T>() * capacity) as isize, null(), usage.convert());
            gl::BindBuffer(_type, 0);
            oid
        };
        Self {
            buffer: oid,
            _type,
            vec,
            usage,
        }
    }
    // Add an element to the raw buffer
    // This may reallocate the OpenGL buffer if it's last len is insufficient
    pub fn push(&mut self, val: T) {
        // Get our old capacity and compare with our new capacity
        let old_capacity = self.vec.capacity();
        // Add the element to our rust vector anyways
        self.vec.push(val);

        // Reallocate the OpenGL buffer if needed
        unsafe {
            gl::BindBuffer(self._type, self.buffer);
            if self.vec.capacity() > old_capacity {
                // Reallocate
                gl::BufferData(self._type, (size_of::<T>() * self.vec.capacity()) as isize, self.vec.as_ptr() as *const c_void, self.usage.convert());
            } else {
                // We don't need to reallocate, we just need to update our sub-data
                let offset = (self.vec.len()-1) * size_of::<T>();
                let data = self.vec.last().unwrap();
                gl::BufferSubData(self._type, offset as isize, size_of::<T>() as isize, data as *const T as *const c_void);
            }
        }   
    }
    // Update a value at a specific index
    pub fn update(&mut self, index: usize, mut function: impl FnMut(&mut T)) {
        // Check first
        if index > self.vec.len() { panic!() }
        // Simple replace 
        let old = self.vec.get_mut(index).unwrap();
        function(old);
        // Also update the OpenGL buffer
        let offset = index * size_of::<T>();
        let data = self.vec.get(index).unwrap();
        unsafe { gl::BufferSubData(self._type, offset as isize, size_of::<T>() as isize, data as *const T as *const c_void); }
    }
    // Replace a value at a specific index
    // This returns the old value at that index
    pub fn replace(&mut self, index: usize, val: T) -> T {
        // Check first
        if index > self.vec.len() { panic!() }
        // Simple replace 
        let old = std::mem::replace(self.vec.get_mut(index).unwrap(), val);
        // Also update the OpenGL buffer
        let offset = index * size_of::<T>();
        let data = self.vec.get(index).unwrap();
        unsafe { gl::BufferSubData(self._type, offset as isize, size_of::<T>() as isize, data as *const T as *const c_void); }
        old
    }
    // Remove an element at a specific index, but by using swap remove, so we don't have to move all the elements
    pub fn swap_remove(&mut self, index: usize) -> T {
        // Check first
        if index > self.vec.len() { panic!() }
        // Simple swap remove
        let old = self.swap_remove(index);
        // Also update the whole OpenGL buffer
        let data = self.vec.as_ptr();
        unsafe { gl::BufferSubData(self._type, 0, (size_of::<T>() * self.vec.len()) as isize, data as *const c_void); }
        old
    }
}