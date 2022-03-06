use std::{ffi::c_void, mem::size_of, ptr::null};

use arrayvec::ArrayVec;
use gl::types::GLuint;

use crate::{
    basics::buffer_operation::BufferOperation,
    object::{OpenGLObjectNotInitialized, PipelineCollectionElement},
    pipeline::{Handle, Pipeline, PipelineCollection},
};

// A simple atomic counter that we can use inside OpenGL fragment and compute shaders, if possible
// This can store multiple atomic counters in a single buffer, thus making it a group
#[derive(Default, Clone)]
pub struct AtomicGroup {
    // The OpenGL ID for the atomic counter buffer
    buffer: GLuint,
}

// Getters
impl AtomicGroup {
    pub(crate) fn buffer(&self) -> GLuint {
        self.buffer
    }
}

impl PipelineCollectionElement for AtomicGroup {
    // Create the OpenGL buffers for this atomic group
    fn added(&mut self, handle: &Handle<Self>) {
        unsafe {
            // Create the OpenGL buffer
            gl::GenBuffers(1, &mut self.buffer);
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, self.buffer);

            // Initialize it's data
            gl::BufferData(gl::ATOMIC_COUNTER_BUFFER, size_of::<u32>() as isize * 4, null(), gl::DYNAMIC_DRAW);

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

impl AtomicGroup {
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
    // Set the atomic group counters
    pub fn set(&mut self, counters: &[u32; 4]) -> Result<(), OpenGLObjectNotInitialized> {
        if self.buffer == 0 {
            return Err(OpenGLObjectNotInitialized);
        }
        unsafe {
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, self.buffer);
            gl::BufferSubData(gl::ATOMIC_COUNTER_BUFFER, 0, size_of::<u32>() as isize * 4, counters.as_ptr() as *const c_void);
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, 0);
        }
        Ok(())
    }
}
