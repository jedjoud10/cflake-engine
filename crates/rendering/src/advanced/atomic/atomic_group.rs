use std::{ffi::c_void, mem::size_of, ptr::null};

use arrayvec::ArrayVec;
use gl::types::GLuint;

use crate::{
    basics::buffer_operation::BufferOperation,
    object::{Construct, ConstructionTask, Deconstruct, DeconstructionTask, GlTracker, ObjectID, OpenGLObjectNotInitialized, PipelineObject},
    pipeline::Pipeline,
};
const MAX_COUNTERS: usize = 4;
// A simple atomic counter that we can use inside OpenGL fragment and compute shaders, if possible
// This can store multiple atomic counters in a single buffer, thus making it a group
#[derive(Clone)]
pub struct AtomicGroup {
    // The OpenGL ID for the atomic counter buffer
    pub(crate) oid: GLuint,
    // Some predefined values that we can set before we execute the shader
    // This also stores the number of valid atomics that we have
    pub(crate) defaults: ArrayVec<u32, MAX_COUNTERS>,
}

impl Default for AtomicGroup {
    fn default() -> Self {
        let mut arrayvec = ArrayVec::<u32, MAX_COUNTERS>::new();
        arrayvec.push(0);
        Self { oid: 0, defaults: arrayvec }
    }
}
impl PipelineObject for AtomicGroup {
    // Reserve an ID for this atomic group
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, pipeline.atomics.gen_id()))
    }
    // Send this atomic group to the pipeline for construction
    fn send(self, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::AtomicGroup(Construct::<Self>(self, id))
    }
    // Create a deconstruction task
    fn pull(id: ObjectID<Self>) -> DeconstructionTask {
        DeconstructionTask::AtomicGroup(Deconstruct::<Self>(id))
    }
    // Add the atomic group to our ordered vec
    fn add(mut self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // Add the atomic group
        // Create the OpenGL atomic buffer
        let mut buffer = 0_u32;
        unsafe {
            gl::GenBuffers(1, &mut buffer);
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, buffer);
            gl::BufferData(
                gl::ATOMIC_COUNTER_BUFFER,
                size_of::<u32>() as isize * self.defaults.len() as isize,
                null(),
                gl::DYNAMIC_DRAW,
            );
            let reset = self.defaults.as_ptr();
            gl::BufferSubData(
                gl::ATOMIC_COUNTER_BUFFER,
                0,
                size_of::<u32>() as isize * self.defaults.len() as isize,
                reset as *const c_void,
            );
            gl::BindBuffer(gl::ATOMIC_COUNTER_BUFFER, 0);
        }
        self.oid = buffer;
        // Add the atomic;
        pipeline.atomics.insert(id, self);
        Some(())
    }
    // Remove the atomic group from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        pipeline.atomics.remove(id)
    }
}

impl AtomicGroup {
    // Create a new atomic counter with some predefined values
    pub fn new(vals: &[u32]) -> Option<Self> {
        let mut arrayvec = ArrayVec::<u32, 4>::new();
        arrayvec.try_extend_from_slice(vals).ok()?;
        Some(Self { oid: 0, defaults: arrayvec })
    }
    // Read/set the value of an atomic group
    pub(crate) fn buffer_operation(&self, op: BufferOperation) -> GlTracker {
        match op {
            BufferOperation::Write(_write) => todo!(),
            BufferOperation::Read(read) => {
                GlTracker::fake(
                    move || unsafe {
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
                    },
                )
            },
        }        
    }
    // Clear the atomic group counters
    pub(crate) fn clear_counters(&self) -> Result<(), OpenGLObjectNotInitialized> {
        if self.oid == 0 {
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
