use crate::{pipeline::Pipeline, utils::UsageType};
use getset::{Getters, MutGetters};
use gl::types::GLuint;
use std::{ffi::c_void, marker::PhantomData, mem::size_of, ptr::null};
// An OpenGL SSBO
#[derive(Getters, MutGetters)]
pub struct ShaderStorage<Buffer: super::raw::Buffer<E>, E> {
    #[getset(get = "pub", get_mut = "pub")]
    storage: Buffer,
    _phantom: PhantomData<E>,
}

impl<Buffer: super::raw::Buffer<E>, E> ShaderStorage<Buffer, E> {
    // Create a new shader storage using a vector
    pub fn new(vec: Vec<E>, usage: UsageType, _pipeline: &Pipeline) -> Self {
        let buffer = Buffer::new_vec(vec, gl::SHADER_STORAGE_BUFFER, usage, _pipeline);
        Self {
            storage: buffer,
            _phantom: PhantomData::default(),
        }
    }
    // Create a new shader storage using a length
    pub fn with_len(len: usize, usage: UsageType, _pipeline: &Pipeline) -> Self {
        let buffer = Buffer::with_len(len, gl::SHADER_STORAGE_BUFFER, usage, _pipeline);
        Self {
            storage: buffer,
            _phantom: PhantomData::default(),
        }
    }
}
