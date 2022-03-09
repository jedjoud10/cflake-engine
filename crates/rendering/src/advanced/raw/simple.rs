use getset::Getters;
use super::{storage::Storage, Buffer};
use crate::{utils::UsageType, pipeline::Pipeline};

// A simple buffer that just holds an OpenGL buffer, but doesn't hold any data by itself
// Can be useful when all we need to do is update some already preallocated buffers, like during terrain generation
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
        Self {
            storage,
        }
    }
    // Read directly from the OpenGL buffer
    fn read(&mut self) -> super::BufferReadGuards<Self, E> {
        todo!()
    }
    // Simple write
    fn write(&mut self, vec: Vec<E>) {
        todo!()
    }
}

impl<E> SimpleBuffer<E> {
    // Set the inner vector
    pub fn set_inner(&mut self, vec: &Vec<E>) {
        // Panic if the sizes don't match
        if self.storage.len() != vec.len() { panic!() }
        self.storage.update(&vec);
    }
}
