use getset::{Getters, MutGetters};
use gl::types::GLuint;
use std::{ffi::c_void, mem::size_of, ptr::null};

use crate::{pipeline::Pipeline, utils::UsageType};

use super::raw::dynamic_buffer::DynamicRawBuffer;

// An OpenGL SSBO
#[derive(Getters, MutGetters)]
pub struct ShaderStorage<T> {
    // Backed by a dynamic raw buffer
    #[getset(get = "pub", get_mut = "pub")]
    storage: DynamicRawBuffer<T>,
}

impl<T> Default for ShaderStorage<T> {
    fn default() -> Self {
        Self { storage: Default::default() }
    }
}

impl<T> ShaderStorage<T> {
    // Create a new empty shader storage
    pub fn new(usage: UsageType, _pipeline: &Pipeline) -> Self {
        Self {
            storage: DynamicRawBuffer::<T>::new(gl::SHADER_STORAGE_BUFFER, usage, _pipeline),
        }
    }
    // Create a new shader storage using some preallocated capacity
    pub fn with_capacity(usage: UsageType, capacity: usize, _pipeline: &Pipeline) -> Self {
        Self {
            storage: DynamicRawBuffer::<T>::with_capacity(gl::SHADER_STORAGE_BUFFER, capacity, usage, _pipeline),
        }
    }
}

// TODO: Drop with disposal of GL data
