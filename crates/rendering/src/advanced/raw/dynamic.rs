use std::{marker::PhantomData, mem::size_of};

use getset::Getters;
use gl::types::GLuint;

use crate::{
    pipeline::Pipeline,
    utils::{AccessType, UsageType},
};

use super::{storage::Storage, Buffer};

// A dynamic buffer that can change in size
// This keeps a Rust copy of the data, so we can add/remove elements to it without the need of reallocating everytime
#[derive(Getters)]
pub struct DynamicBuffer<E> {
    // Storage
    storage: Storage<E>,
    // Rust vector
    #[getset(get = "pub")]
    inner: Vec<E>,
}

// Creation
impl<E> Buffer<E> for DynamicBuffer<E> {
    // Storage
    fn storage(&self) -> &Storage<E> {
        &self.storage
    }
    // Create a dynamic buffer
    fn new_raw(cap: usize, len: usize, ptr: *const E, _type: GLuint, usage: UsageType, _pipeline: &Pipeline) -> Self {
        // Dynamic buffer cannot be for buffers that have an AccessType of ServerToClient or ServerToServer, because we don't know when we have update the buffer on the GPU
        match usage.access {
            AccessType::ServerToServer | AccessType::ServerToClient => panic!(),
            _ => (),
        }
        let mut storage = Storage::new(_type, usage, _pipeline);
        // Fill the storage
        storage.reallocate(ptr, cap);
        Self { storage, inner: {
            let mut vec = Vec::with_capacity(cap);
            
            // Fill if possible
            if !ptr.is_null() && len > 0{
                unsafe { 
                    let slice = std::slice::from_raw_parts(ptr, len);
                    std::ptr::copy(slice.as_ptr(), vec.as_mut_ptr(), len);
                };
            }
            
            vec
        }}
    }
    // Read from the dynamic buffer
    // This will actually read from the OpenGL buffer, then store it internally, and return a reference to that
    fn read(&mut self, output: &mut [E]) {
        // Map the buffer
        let ptr = unsafe {
            let ptr = gl::MapNamedBuffer(self.storage.buffer(), gl::READ_ONLY);
            // Check validity
            if ptr.is_null() {
                panic!()
            }
            ptr
        };
        // Read the whole buffer slice from the pointer
        let len = self.inner.len() * size_of::<E>();

        // Store internally first
        unsafe { std::ptr::copy(ptr as *const E, self.inner.as_mut_ptr(), len) }

        // Then copy to output
        unsafe { std::ptr::copy(ptr as *const E, output.as_mut_ptr(), len) }

        // We can unmap the buffer now
        unsafe {
            let result = gl::UnmapNamedBuffer(self.storage.buffer());
        }
    }
    // Simple write
    // The push and pop commands automatically write to the buffer as well
    fn write(&mut self, vec: Vec<E>) {
        self.inner = vec;
        self.storage.update(&self.inner);
    }
}

// Push, set, pop, len
impl<E> DynamicBuffer<E> {
    // Push a single element
    pub fn push(&mut self, value: E) {
        self.inner.push(value);
        self.storage.update(&self.inner);
    }
    // Pop a single element
    pub fn pop(&mut self) {
        self.inner.pop();
        // Popping won'E deallocate, so we don'E have to do anything
    }
    // Length and is_empty
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
