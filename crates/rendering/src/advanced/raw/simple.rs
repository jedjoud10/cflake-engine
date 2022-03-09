use std::{marker::PhantomData, mem::size_of};

use super::{storage::Storage, Buffer};
use crate::{pipeline::Pipeline, utils::UsageType};
use getset::Getters;

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
    fn new(vec: Vec<E>, _type: u32, usage: UsageType, _pipeline: &Pipeline) -> Self {
        let mut storage = Storage::new(_type, usage, _pipeline);
        // Fill the storage
        storage.reallocate(&vec);
        Self { storage }
    }
    // Read directly from the OpenGL buffer
    fn read(&mut self, output: &mut [E]) {
        // Map the buffer
        let ptr = unsafe {
            let ptr = gl::MapNamedBuffer(self.storage.buffer(), gl::MAP_READ_BIT);
            // Check validity
            if ptr.is_null() {
                panic!()
            }
            ptr
        };
        // Read the whole buffer slice from the pointer
        let len = self.storage.len() * size_of::<E>();

        // Then copy to output
        unsafe { std::ptr::copy(ptr as *const E, output.as_mut_ptr(), len) }

        // We can unmap the buffer now
        unsafe {
            let result = gl::UnmapNamedBuffer(self.storage.buffer());
        }
    }
    // Simple write
    fn write(&mut self, vec: Vec<E>) {
        // Panic if the sizes don't match
        if self.storage.len() != vec.len() {
            panic!()
        }
        self.storage.update(&vec);
    }
}
