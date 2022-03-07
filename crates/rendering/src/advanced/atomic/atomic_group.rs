use std::{ffi::c_void, mem::size_of, ptr::null};

use arrayvec::ArrayVec;
use getset::{CopyGetters, Getters};
use gl::types::GLuint;

use crate::{
    advanced::tracker::{GlTracker, MaybeGlTracker},
    basics::bufop::{Readable, Writable},
    object::{OpenGLObjectNotInitialized, PipelineCollectionElement},
    pipeline::{Handle, Pipeline, PipelineCollection},
};

// Le array
pub type AtomicArray = [u32; 4];

// A simple atomic counter that we can use inside OpenGL fragment and compute shaders, if possible
// This can store multiple atomic counters in a single buffer, thus making it a group
#[derive(Getters, CopyGetters, Default, Clone)]
pub struct AtomicGroup {
    // The OpenGL ID for the atomic counter buffer
    #[getset(get_copy = "pub")]
    buffer: GLuint,
    #[getset(get = "pub")]
    array: AtomicArray,
}

impl PipelineCollectionElement for AtomicGroup {
    // Create the OpenGL buffers for this atomic group
    fn added(&mut self, handle: &Handle<Self>) {
        unsafe {
            // Create the OpenGL buffer
            gl::GenBuffers(1, &mut self.buffer);
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, self.buffer);

            // Initialize it's data
            gl::BufferData(gl::ATOMIC_COUNTER_BUFFER, size_of::<AtomicArray>() as isize, null(), gl::DYNAMIC_DRAW);

            // Unbind just in case
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, 0);
        }
    }

    // Dispose of the OpenGL buffers
    fn disposed(mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.buffer);
        }
    }
}

impl Writable for AtomicGroup {
    type Data = AtomicArray;

    fn glwrite(&mut self, input: Self::Data) -> MaybeGlTracker<Self, ()> {
        // Check validity
        if self.buffer == 0 {
            return Err(OpenGLObjectNotInitialized);
        }
        // Write to the atomic counter
        Ok(GlTracker::new(|| unsafe {
            // Set the values
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, self.buffer);
            gl::BufferSubData(gl::ATOMIC_COUNTER_BUFFER, 0, size_of::<AtomicArray>() as isize, self.array.as_ptr() as *mut c_void);
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, 0);
            &()
        }))
    }
}

impl Readable for AtomicGroup {
    type Data = AtomicArray;

    fn glread(&mut self) -> MaybeGlTracker<Self, Self::Data> {
        // Check validity
        if self.buffer == 0 {
            return Err(OpenGLObjectNotInitialized);
        }
        // Read the atomic counter
        Ok(GlTracker::new(|| unsafe {
            // Read the values
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, self.buffer);
            gl::GetBufferSubData(gl::ATOMIC_COUNTER_BUFFER, 0, size_of::<AtomicArray>() as isize, self.array.as_mut_ptr() as *mut c_void);
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, 0);
            // Success
            &self.array
        }))
    }
}
