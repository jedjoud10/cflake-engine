use std::{marker::PhantomData, mem::{size_of, MaybeUninit}, ptr::null};

use super::{storage::Storage, Buffer};
use crate::{pipeline::Pipeline, utils::UsageType};
use getset::Getters;
use gl::types::GLuint;

// A simple buffer that just holds an OpenGL buffer, but doesn't hold any data by itself
// Can be useful when all we need to do is update some already preallocated buffers
#[derive(Getters)]
pub struct SimpleBuffer<Element> {
    // Storage
    storage: Storage<Element>,
}

// Creation
impl<Element> Buffer for SimpleBuffer<Element> {
    type Element = Element;
    // Storage
    fn storage(&self) -> &Storage<Element> {
        &self.storage
    }
    // Create a simple buffer THAT CANNOT CHANGE SIZE
    unsafe fn new_raw(_cap: usize, len: usize, ptr: *const Element, _type: GLuint, usage: UsageType, _pipeline: &Pipeline) -> Self {
        // Init and fill
        let storage = Storage::new(len, len, ptr, _type, usage);
        Self { storage }
    }
    // Read directly from the OpenGL buffer
    fn read(&mut self, output: &mut [Element]) {
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
        unsafe { std::ptr::copy(ptr as *const Element, output.as_mut_ptr(), len) }

        // We can unmap the buffer now
        unsafe {
            let result = gl::UnmapBuffer(self.storage._type());
        }
    }
    // Simple write
    fn write(&mut self, buf: &[Element])
    where
        Element: Copy,
    {
        // Panic if the sizes don't match
        if self.storage.len() != buf.len() {
            panic!("Length mismatch, src length is '{}', new vec length is '{}'", self.storage.len(), buf.len());
        }
        self.storage.update(buf.as_ptr(), buf.len(), buf.len());
    }
    // With capacity (also set the buffer's length)
    fn with_capacity(capacity: usize, _type: GLuint, usage: UsageType, _pipeline: &Pipeline) -> Self {
        unsafe { Self::new_raw(capacity, capacity, null(), _type, usage, _pipeline) }
    }
}
