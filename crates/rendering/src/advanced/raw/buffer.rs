use std::{ffi::c_void, marker::PhantomData, ptr::null};

use gl::types::GLuint;

use crate::{pipeline::Pipeline, utils::UsageType};

use super::storage::Storage;

// Buffer trait
pub trait Buffer<Element>
where
    Self: Sized,
{
    // Get the raw Storage
    fn storage(&self) -> &Storage<Element>;
    // Create a new buffer using some capacity, length, and a pointer
    fn new_raw(cap: usize, len: usize, ptr: *const Element, _type: GLuint, usage: UsageType, _pipeline: &Pipeline) -> Self;
    // Create a new buffer using a vector
    fn new_vec(vec: Vec<Element>, _type: GLuint, usage: UsageType, _pipeline: &Pipeline) -> Self {
        let ptr = vec.as_ptr();
        Self::new_raw(vec.capacity(), vec.len(), ptr, _type, usage, _pipeline)
    }
    // Create a new uninitialized buffer with length len
    fn with_len(len: usize, _type: GLuint, usage: UsageType, _pipeline: &Pipeline) -> Self {
        Self::new_raw(len, len, null(), _type, usage, _pipeline)
    }
    // Get the underlying data
    fn read(&mut self, output: &mut [Element]);
    // Set the underlying data
    fn write(&mut self, vec: Vec<Element>);
}
