use std::marker::PhantomData;

use gl::types::GLuint;

use crate::{utils::UsageType, pipeline::Pipeline};

use super::storage::Storage;


// Buffer trait
pub trait Buffer<Element> where Self: Sized {
    // Get the raw Storage
    fn storage(&self) -> &Storage<Element>;
    // Create a new buffer
    fn new(vec: Vec<Element>, _type: u32, usage: UsageType, _pipeline: &Pipeline) -> Self;
    // Get the underlying data
    fn read(&mut self) -> BufferReadGuards<Self, Element>;
    // Set the underlying data
    fn write(&mut self, vec: Vec<Element>); 
}

// Buffer read guards
// B: Buffer
// E: Element
pub struct BufferReadGuards<'a, B, E> {
    // Since we will be mapping the buffers to access them, we must make sure that we are not writing to said buffers while we are reading them
    buffer: &'a mut B,
    _phantom: PhantomData<*const E>,
}

impl<'a, B: Buffer<E>, E: 'a> std::ops::Deref for BufferReadGuards<'a, B, E> {
    type Target = &'a [E];

    fn deref(&self) -> &Self::Target {
        todo!()
    }
}

