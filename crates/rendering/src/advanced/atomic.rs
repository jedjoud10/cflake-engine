use std::{
    ffi::c_void,
    mem::{size_of, MaybeUninit},
    ptr::null,
};

use arrayvec::ArrayVec;
use getset::{CopyGetters, Getters};
use gl::types::GLuint;

use crate::{
    object::{OpenGLObjectNotInitialized, PipelineCollectionElement},
    pipeline::{Handle, Pipeline, PipelineCollection},
    utils::UsageType,
};

use super::raw::{simple::SimpleBuffer, Buffer};

// Le array
pub const ATOMIC_COUNTERS_NUM: usize = 4;
pub type AtomicArray = [u32; ATOMIC_COUNTERS_NUM];

// A simple atomic counter that we can use inside OpenGL fragment and compute shaders, if possible
// This can store multiple atomic counters in a single buffer, thus making it a group
#[derive(Getters, CopyGetters)]
pub struct AtomicGroup {
    // Backed by a simple buffer
    #[getset(get = "pub")]
    storage: SimpleBuffer<u32>,
}

impl AtomicGroup {
    // New empty atomic group
    pub fn new(usage: UsageType, _pipeline: &Pipeline) -> Self {
        let arr = AtomicArray::default();
        Self {
            storage: unsafe { SimpleBuffer::new_raw(ATOMIC_COUNTERS_NUM, ATOMIC_COUNTERS_NUM, arr.as_ptr(), gl::ATOMIC_COUNTER_BUFFER, usage, _pipeline) },
        }
    }
    // Wrapper functions around the inner storage

    // Set the atomic group's values
    pub fn set(&mut self, arr: AtomicArray) {
        self.storage.write(&arr);
    }
    // Read the atomic group's values
    pub fn get(&mut self) -> AtomicArray {
        let mut output = AtomicArray::default();
        self.storage.read(&mut output);
        output
    }
}
