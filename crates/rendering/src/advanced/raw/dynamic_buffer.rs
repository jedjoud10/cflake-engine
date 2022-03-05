use gl::types::GLuint;
use crate::{utils::UsageType, pipeline::Pipeline};
use std::{ffi::c_void, mem::size_of, ptr::null, marker::PhantomData};

// A dynamic OpenGL buffer that automatically reallocates it's size when we add too many elements to it
pub struct DynamicRawBuffer<T> {
    // The OpenGL data for this buffer
    buffer: GLuint,
    _type: GLuint,

    // Other data
    usage: UsageType,
    inner: Vec<T>,
    _phantom: PhantomData<*const ()>,
}

impl<T> DynamicRawBuffer<T> {
    
}

impl<T> DynamicRawBuffer<T> {
    // Create the dynamic raw buffer
    pub fn new(_type: u32, usage: UsageType, _pipeline: &Pipeline) -> Self {
        Self::with_capacity(_type, 0, usage, _pipeline)
    }
    // Create a new dynamic raw buffer with a specified capacity
    pub fn with_capacity(_type: u32, capacity: usize, usage: UsageType, _pipeline: &Pipeline) -> Self {
        let vec = Vec::<T>::with_capacity(capacity);
        let oid = unsafe {
            let mut oid = 0;
            gl::GenBuffers(1, &mut oid);
            gl::BindBuffer(_type, oid);
            if capacity != 0 {
                gl::BufferData(_type, (size_of::<T>() * capacity) as isize, null(), usage.convert());
            }
            gl::BindBuffer(_type, 0);
            oid
        };
        Self { buffer: oid, _type, inner: vec, usage, _phantom: PhantomData::default() }
    }
    // Set the contents of the dynamic raw buffer from an already allocated vector
    pub fn set_contents(&mut self, vec: Vec<T>) {
        unsafe {
            self.inner = vec;
            gl::BindBuffer(self._type, self.buffer);
            gl::BufferData(
                self._type,
                (size_of::<T>() * self.inner.len()) as isize,
                self.inner.as_ptr() as *const c_void,
                self.usage.convert(),
            );
            gl::BindBuffer(self._type, 0);
        }
    }
    // Length and is_empty
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}


impl<T> Drop for DynamicRawBuffer<T> {
    fn drop(&mut self) {
        // Dispose of the OpenGL buffer
        if self.buffer != 0 {
            unsafe {
                gl::DeleteBuffers(1, &mut self.buffer);
            }
        } 
    }
}