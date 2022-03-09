use getset::{Getters, MutGetters};
use gl::types::GLuint;
use std::{ffi::c_void, mem::size_of, ptr::null, marker::PhantomData};
use crate::{pipeline::Pipeline, utils::UsageType};
// An OpenGL SSBO
#[derive(Getters, MutGetters)]
pub struct ShaderStorage<Buffer: super::raw::Buffer<E>, E> {
    #[getset(get = "pub", get_mut = "pub")]
    storage: Buffer,
    _phantom: PhantomData<E>,
}

impl<Buffer: super::raw::Buffer<E>, E> ShaderStorage<Buffer, E> {
    // Create a new shader storage
    pub fn new(vec: Vec<E>, _type: u32, usage: UsageType, _pipeline: &Pipeline) -> Self {
        let buffer = Buffer::new(vec, _type, usage, _pipeline);
        Self { storage: buffer, _phantom: PhantomData::default() }
    }
}