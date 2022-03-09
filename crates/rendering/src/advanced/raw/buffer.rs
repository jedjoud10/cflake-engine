use std::{ffi::c_void, marker::PhantomData};

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
    // Create a new buffer
    fn new(vec: Vec<Element>, _type: GLuint, usage: UsageType, _pipeline: &Pipeline) -> Self;
    // Get the underlying data
    fn read(&mut self, output: &mut [Element]);
    // Set the underlying data
    fn write(&mut self, vec: Vec<Element>);
}
