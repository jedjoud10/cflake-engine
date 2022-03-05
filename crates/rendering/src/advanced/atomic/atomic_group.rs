use std::{ffi::c_void, mem::size_of, ptr::null};

use arrayvec::ArrayVec;
use gl::types::GLuint;

use crate::{
    basics::buffer_operation::BufferOperation,
    object::{OpenGLObjectNotInitialized, OpenGLInitializer},
    pipeline::{Pipeline, PipelineCollection, Handle},
};

// A simple atomic counter that we can use inside OpenGL fragment and compute shaders, if possible
// This can store multiple atomic counters in a single buffer, thus making it a group
#[derive(Default, Clone)]
pub struct AtomicGroup {
    // The OpenGL ID for the atomic counter buffer
    pub(crate) buffer: GLuint,
    // Some predefined values that we can set before we execute the shader
    pub(crate) defaults: ArrayVec<u32, 4>,
}

impl OpenGLInitializer for AtomicGroup {
    // Create the OpenGL buffers for this atomic group
    fn added(&mut self, collection: &mut PipelineCollection<Self>, handle: Handle<Self>) {
        self.buffer = unsafe {
            // Create the OpenGL buffer
            let mut buffer = 0u32;
            gl::GenBuffers(1, &mut buffer);
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, buffer);

            // Initialize it's data
            gl::BufferData(
                gl::ATOMIC_COUNTER_BUFFER,
                size_of::<u32>() as isize * self.defaults.len() as isize,
                self.defaults.as_ptr() as *const c_void,
                gl::DYNAMIC_DRAW,
            );

            // Unbind just in case
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, 0);
        }
    }
}

impl AtomicGroup {
    // Create a new atomic counter with some predefined values
    pub fn new(vals: &[u32]) -> Option<Self> {
        let mut arrayvec = ArrayVec::<u32, 4>::new();
        arrayvec.try_extend_from_slice(vals).ok()?;
        Some(Self { buffer: 0, defaults: arrayvec })
    }
    /*
    // Read/set the value of an atomic group
    pub fn buffer_operation(&self, op: BufferOperation) -> GlTracker {
        match op {
            BufferOperation::Write(_write) => todo!(),
            BufferOperation::Read(read) => {
                GlTracker::fake(move || unsafe {
                    // Read the value of the atomics from the buffer, and update the shared Transfer<AtomicGroupRead>'s inner value
                    let oid = self.oid;
                    let mut bytes: Vec<u8> = vec![0; self.defaults.len() * size_of::<u32>()];
                    gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, oid);
                    gl::GetBufferSubData(
                        gl::ATOMIC_COUNTER_BUFFER,
                        0,
                        size_of::<u32>() as isize * self.defaults.len() as isize,
                        bytes.as_mut_ptr() as *mut c_void,
                    );
                    gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, 0);
                    // Now store the atomic counter's values
                    let mut cpu_counters_lock = read.bytes.lock();
                    let cpu_counters = &mut *cpu_counters_lock;
                    *cpu_counters = bytes;
                })
            }
        }
    }
    */
    // Clear the atomic group counters
    pub(crate) fn clear_counters(&self) -> Result<(), OpenGLObjectNotInitialized> {
        if self.buffer {
            return Err(OpenGLObjectNotInitialized);
        }
        unsafe {
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, self.oid);
            let reset = self.defaults.as_ptr();
            gl::BufferSubData(
                gl::ATOMIC_COUNTER_BUFFER,
                0,
                size_of::<u32>() as isize * self.defaults.len() as isize,
                reset as *const c_void,
            );
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, 0);
        }
        Ok(())
    }
}
