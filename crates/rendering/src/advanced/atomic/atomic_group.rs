use std::{ffi::c_void, mem::size_of, ptr::null};

use arrayvec::ArrayVec;
use getset::{CopyGetters, Getters};
use gl::types::GLuint;

use crate::{
    basics::bufop::GLBufferOperations,
    object::{OpenGLObjectNotInitialized, PipelineCollectionElement},
    pipeline::{Handle, Pipeline, PipelineCollection}, utils::UsageType,
};

// Le array
pub type AtomicArray = [u32; 4];

// A simple atomic counter that we can use inside OpenGL fragment and compute shaders, if possible
// This can store multiple atomic counters in a single buffer, thus making it a group
#[derive(Getters, CopyGetters, Clone)]
pub struct AtomicGroup {
    // The OpenGL ID for the atomic counter buffer
    #[getset(get_copy = "pub")]
    buffer: GLuint,
    #[getset(get = "pub")]
    array: AtomicArray,
}

impl AtomicGroup {
    // New
    pub fn new(usage: UsageType, _pipeline: &Pipeline) -> Self {
        let mut buffer = 0;
        unsafe {
            // Create the OpenGL buffer
            gl::GenBuffers(1, &mut buffer);
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, buffer);

            // Initialize it's data
            gl::BufferData(gl::ATOMIC_COUNTER_BUFFER, size_of::<AtomicArray>() as isize, null(), usage.convert());

            // Unbind just in case
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, 0);
        }
        Self {
            buffer: buffer,
            array: Default::default(),
        }
    }
}

impl GLBufferOperations for AtomicGroup {
    type Data = AtomicArray;
    fn glupdate(&mut self) -> Result<(), OpenGLObjectNotInitialized> {
        // Check validity
        if self.buffer == 0 {
            return Err(OpenGLObjectNotInitialized);
        }
        unsafe {
            // Set the values
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, self.buffer);
            gl::BufferSubData(gl::ATOMIC_COUNTER_BUFFER, 0, size_of::<AtomicArray>() as isize, self.array.as_ptr() as *mut c_void);
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, 0);
            Ok(())
        }
    }
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
        self.glupdate()?;
        Ok(())
    }
}
