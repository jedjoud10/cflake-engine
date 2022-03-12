use std::ptr::null;

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
        let mut usage = usage;
        let storage = Storage::new(len, len, ptr, _type, usage);
        Self { storage }
    }
    // Read directly from the OpenGL buffer
    fn read(&mut self, output: &mut [Element]) {
        unsafe { self.storage.read_subdata(output.as_mut_ptr(), self.storage().len(), 0) }
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
