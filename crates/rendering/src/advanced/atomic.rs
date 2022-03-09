use std::{ffi::c_void, mem::size_of, ptr::null};

use arrayvec::ArrayVec;
use getset::{CopyGetters, Getters};
use gl::types::GLuint;

use crate::{
    basics::mapper::{MappableGLBuffer, MappedBufferReader, MappedBufferWriter},
    object::{OpenGLObjectNotInitialized, PipelineCollectionElement},
    pipeline::{Handle, Pipeline, PipelineCollection},
    utils::UsageType,
};

use super::raw::{simple::SimpleBuffer, Buffer};

// Le array
pub type AtomicArray = [u32; 4];

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
        Self {
            storage: SimpleBuffer::with_len(4, gl::ATOMIC_COUNTER_BUFFER, usage, _pipeline),
        }
    }
    // Wrapper functions around the inner storage

    // Set the atomic group's values
    pub fn set(&mut self, arr: AtomicArray) {
        // TODO: Remove this vector allocation
        self.storage.write(Vec::from(arr));
    }
    // Read the atomic group's values
    pub fn get(&mut self) -> AtomicArray {
        let mut output = AtomicArray::default();
        self.storage.read(&mut output);
        output
    }
}

/*
impl GLBufferOperations for AtomicGroup {
    type Data = AtomicArray;
    fn glread(&mut self) -> Result<&Self::Data, OpenGLObjectNotInitialized> {
        // Check validity
        if self.buffer == 0 {
            return Err(OpenGLObjectNotInitialized);
        }
        unsafe {
            // Read the values
            gl::MemoryBarrier(gl::ATOMIC_COUNTER_BARRIER_BIT);
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, self.buffer);
            gl::GetBufferSubData(gl::ATOMIC_COUNTER_BUFFER, 0, size_of::<AtomicArray>() as isize, self.array.as_mut_ptr() as *mut c_void);
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, 0);
            // Success
            Ok(&self.array)
        }
    }
    fn glset(&mut self, data: Self::Data) -> Result<(), OpenGLObjectNotInitialized> {
        // Check validity
        if self.buffer == 0 {
            return Err(OpenGLObjectNotInitialized);
        }
        self.array = data;
        unsafe {
            // Set the values
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, self.buffer);
            gl::BufferSubData(gl::ATOMIC_COUNTER_BUFFER, 0, size_of::<AtomicArray>() as isize, self.array.as_ptr() as *mut c_void);
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, 0);
            Ok(())
        }
    }
}
*/
