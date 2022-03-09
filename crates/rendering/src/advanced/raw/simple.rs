use std::{marker::PhantomData, mem::size_of};

use super::{storage::Storage, Buffer};
use crate::{pipeline::Pipeline, utils::UsageType};
use getset::Getters;
use gl::types::GLuint;

// A simple buffer that just holds an OpenGL buffer, but doesn't hold any data by itself
// Can be useful when all we need to do is update some already preallocated buffers
#[derive(Getters)]
pub struct SimpleBuffer<E> {
    // Storage
    storage: Storage<E>,
}

// Creation
impl<E> Buffer<E> for SimpleBuffer<E> {
    // Storage
    fn storage(&self) -> &Storage<E> {
        &self.storage
    }
    // Create a simple buffer THAT CANNOT CHANGE SIZE
    fn new_raw(_cap: usize, len: usize, ptr: *const E, _type: GLuint, usage: UsageType, _pipeline: &Pipeline) -> Self {
        let mut storage = Storage::new(_type, usage, _pipeline);
        // Fill the storage
        storage.init(len, len, ptr);
        Self { storage }
    }
    // Read directly from the OpenGL buffer
    fn read(&mut self, output: &mut [E]) {
        // Map the buffer
        let ptr = unsafe {
            gl::BindBuffer(self.storage._type(), self.storage.buffer());
            let ptr = gl::MapBuffer(self.storage._type(), gl::READ_ONLY);
            // Check validity
            if ptr.is_null() {
                panic!()
            }
            ptr
        };
        // Read the whole buffer slice from the pointer
        let len = self.storage.len();

        // Then copy to output
        unsafe { std::ptr::copy(ptr as *const E, output.as_mut_ptr(), len) }

        // We can unmap the buffer now
        unsafe {
            let result = gl::UnmapBuffer(self.storage._type());
        }
    }
    // Simple write
    fn write(&mut self, vec: Vec<E>) {
        // Panic if the sizes don't match
        if self.storage.len() != vec.len() {
            panic!("Length mismatch, src length is '{}', new vec length is '{}'", self.storage.len(), vec.len());
        }
        self.storage.update(&vec);
    }
}
