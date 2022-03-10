use crate::{pipeline::Pipeline, utils::UsageType};
use getset::{Getters, MutGetters};
use gl::types::GLuint;
use std::{ffi::c_void, marker::PhantomData, mem::{size_of, MaybeUninit}, ptr::null};
// An OpenGL SSBO
#[derive(Getters, MutGetters)]
pub struct ShaderStorage<Buffer: super::raw::Buffer> {
    #[getset(get = "pub", get_mut = "pub")]
    storage: Buffer,
}

impl<Buffer: super::raw::Buffer> ShaderStorage<Buffer> {
    // Create a new empty shader storage
    pub fn new_empty(usage: UsageType, _pipeline: &Pipeline) -> Self {
        Self {
            storage: Buffer::empty(gl::SHADER_STORAGE_BUFFER, usage, _pipeline)
        }
    }
    // Create a new shader storage from raw data
    pub unsafe fn new_raw(cap: usize, len: usize, ptr: *const Buffer::Element, usage: UsageType, _pipeline: &Pipeline) -> Self {
        Self {
            storage: Buffer::new_raw(cap, len, ptr, gl::SHADER_STORAGE_BUFFER, usage, _pipeline),
        }
    }
    // Create a new shader storage using a vector of unitialized data
    pub fn new(vec: Vec<MaybeUninit<Buffer::Element>>, usage: UsageType, _pipeline: &Pipeline) -> Self {
        Self {
            storage: Buffer::new(vec, gl::SHADER_STORAGE_BUFFER, usage, _pipeline),
        }
    }
    // Create a new buffer using a specified capacity
    pub fn with_capacity(capacity: usize, usage: UsageType, _pipeline: &Pipeline) -> Self {
        Self {
            storage: Buffer::with_capacity(capacity, gl::SHADER_STORAGE_BUFFER, usage, _pipeline),
        }
    }
}
