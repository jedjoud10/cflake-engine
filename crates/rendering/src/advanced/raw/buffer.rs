use std::{ffi::c_void, marker::PhantomData, ptr::null, mem::MaybeUninit};

use gl::types::GLuint;

use crate::{pipeline::Pipeline, utils::UsageType};

use super::storage::Storage;

// Buffer trait
pub trait Buffer
where
    Self: Sized,
{
    // Inner storage element
    type Element;

    // Get the raw Storage
    fn storage(&self) -> &Storage<Self::Element>;
    // Create a new buffer using some capacity, length, and a pointer
    unsafe fn new_raw(cap: usize, len: usize, ptr: *const Self::Element, _type: GLuint, usage: UsageType, _pipeline: &Pipeline) -> Self;
    // Create a new empty vector
    fn empty(_type: GLuint, usage: UsageType, _pipeline: &Pipeline) -> Self {
        unsafe {
            Self::new_raw(0, 0, null(), _type, usage, _pipeline)
        }
    }
    // Create a new buffer using a vector
    fn new(vec: Vec<MaybeUninit<Self::Element>>, _type: GLuint, usage: UsageType, _pipeline: &Pipeline) -> Self {
        let ptr = vec.as_ptr() as *const Self::Element;
        unsafe {
            Self::new_raw(vec.capacity(), vec.len(), ptr, _type, usage, _pipeline)
        }
    }
    // Create a new buffer using a specified capacity
    fn with_capacity(capacity: usize, _type: GLuint, usage: UsageType, _pipeline: &Pipeline) -> Self {
        unsafe {
            Self::new_raw(capacity, 0, null(), _type, usage, _pipeline)
        }
    }
    // Get the underlying data
    fn read(&mut self, buf: &mut [Self::Element]);
    // Set the underlying data
    // This will take all the values out of buf and store them into the OpenGL buffer 
    fn write(&mut self, buf: &[Self::Element]) where Self::Element: Copy;
}
