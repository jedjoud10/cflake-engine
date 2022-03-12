use getset::Getters;
use gl::types::GLuint;

use crate::{
    pipeline::Pipeline,
    utils::{AccessType, UsageType},
};

use super::{storage::TypedStorage, Buffer};

// A dynamic buffer that can change in size
// This keeps a Rust copy of the data, so we can add/remove elements to it without the need of reallocating everytime
#[derive(Getters)]
pub struct DynamicBuffer<Element> {
    // Storage
    storage: TypedStorage<Element>,
    // Rust vector
    #[getset(get = "pub")]
    inner: Vec<Element>,
}

// Creation
impl<Element> Buffer for DynamicBuffer<Element> {
    type Element = Element;
    // Buffer
    fn buffer(&self) -> GLuint { self.storage.buffer() }
    // Storage
    fn storage(&self) -> &TypedStorage<Element> {
        &self.storage
    }
    // Create a dynamic buffer
    unsafe fn new_raw(cap: usize, len: usize, ptr: *const Element, _type: GLuint, usage: UsageType, _pipeline: &Pipeline) -> Self {
        // Dynamic buffer cannot be for buffers that have an AccessType of ServerToClient or ServerToServer, because we don't know when we have update the buffer on the GPU
        match usage.access {
            AccessType::ServerToServer | AccessType::ServerToClient => panic!(),
            _ => (),
        }
        // Init and fill
        let storage = TypedStorage::new(cap, len, ptr, _type, usage);

        Self {
            storage,
            inner: {
                let mut vec = Vec::with_capacity(cap);
                // Fill if possible
                if !ptr.is_null() && len > 0 {
                    let slice = std::slice::from_raw_parts(ptr, len);
                    std::ptr::copy(slice.as_ptr(), vec.as_mut_ptr(), len);
                }
                vec
            },
        }
    }
    // Read from the dynamic buffer
    // This will actually read from the OpenGL buffer, then store it internally, and return a reference to that
    fn read(&mut self, output: &mut [Element]) {
        // Store internally first
        unsafe { self.storage.read(self.inner.as_mut_ptr(), self.storage().len(), 0) }

        // Then copy to output
        unsafe { std::ptr::copy(self.inner.as_ptr() as *const Element, output.as_mut_ptr(), self.storage().len()) }
    }
    // Simple write
    // The push and pop commands automatically write to the buffer as well
    fn write(&mut self, buf: &[Element])
    where
        Element: Copy,
    {
        self.inner.clear();
        self.inner.extend_from_slice(buf);
        unsafe { self.storage.update(self.inner.as_ptr(), self.capacity(), self.len()); }
    }
}

// Push, set, pop, len
impl<Element> DynamicBuffer<Element> {
    // Push a single element
    pub fn push(&mut self, value: Element) {
        self.inner.push(value);
        self.storage.update(self.inner.as_ptr(), self.capacity(), self.len());
    }
    // Pop a single element
    pub fn pop(&mut self) {
        self.inner.pop();
        // Popping won't deallocate, so we don't have to do anything
    }
    // Capacity
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }
    // Length and is_empty
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
