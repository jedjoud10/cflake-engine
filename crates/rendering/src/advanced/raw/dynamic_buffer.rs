use gl::types::GLuint;
use crate::{utils::UsageType, pipeline::Pipeline};
use std::{ffi::c_void, mem::size_of, ptr::null, marker::PhantomData, ops::Range};

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

// Getters
impl<T> DynamicRawBuffer<T> {
    pub(crate) fn buffer(&self) -> GLuint { self.buffer }
    pub fn usage(&self) -> UsageType { self.usage }
    pub fn inner(&self) -> &Vec<T> { &self.inner }
}

// Creation
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
}

// Allocation
impl<T> DynamicRawBuffer<T> {
    // Reallocate the buffer
    fn reallocate(&self) {
        unsafe {
            gl::BindBuffer(self._type, self.buffer);
            gl::BufferData(
                self._type,
                (size_of::<T>() * self.inner.capacity()) as isize,
                self.inner.as_ptr() as *const c_void,
                self.usage.convert(),
            );
            gl::BindBuffer(self._type, 0);
        }
    }
    // Update the buffer data in the specified range (elements)
    fn update(&self, range: Range<usize>) {
        unsafe {
            gl::BindBuffer(self._type, self.buffer);
            // Borth range indices correspond to elements' indices, not their byte offset
            let start = range.start;
            let end = range.end;

            // Byte offset and byte size
            let byte_offset = start * size_of::<T>();
            let byte_size = (end - start) * size_of::<T>();
            gl::BufferSubData(
                self._type,
                byte_offset as isize,
                byte_size as isize,
                self.inner.as_ptr() as *const c_void,
            );
            gl::BindBuffer(self._type, 0);
        }
    }
}

// Push, set, pop, len
impl<T> DynamicRawBuffer<T> {
    // Set the contents of the dynamic raw buffer from an already allocated vector
    pub fn set_inner(&mut self, vec: Vec<T>) {
        // Completely reallocate
        self.inner = vec;
        self.reallocate();
    }

    // Push a single element
    pub fn push(&mut self, value: T) {
        let old = self.inner.capacity();
        self.inner.push(value);
        let new = self.inner.capacity();
        let index = self.inner.len() - 1;
    
        // Reallocate only if we exceed the old capacity 
        if new > old {
            self.reallocate();
        } else {
            // Otherwise, just update
            self.update(index..(index+1))
        }
    }
    // Pop a single element
    pub fn pop(&mut self) {
        self.inner.pop();    }

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