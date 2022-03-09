use getset::Getters;

use crate::{utils::UsageType, pipeline::Pipeline};

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
    fn new(vec: Vec<E>, _type: u32, usage: UsageType, _pipeline: &Pipeline) -> Self {
        let mut storage = Storage::new(_type, usage, _pipeline);
        // Fill the storage
        storage.reallocate(&vec);
        Self {
            storage,
            inner: vec,
        }
    }
    // Read from the dynamic buffer
    // This will actually read from the OpenGL buffer, then store it internally, and return a reference to that
    fn read(&mut self) -> super::BufferReadGuards<Self, E> {
        todo!()
    }
    // Simple write
    fn write(&mut self, vec: Vec<E>) {
        todo!()
    }    
}

// Push, set, pop, len
impl<E> DynamicBuffer<E> {
    // Set the inner vector
    pub fn set_inner(&mut self, vec: Vec<E>) {
        self.inner = vec;
        self.storage.update(&self.inner);
    }
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
