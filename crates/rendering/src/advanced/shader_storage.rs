use std::{ffi::c_void, ptr::null};

use crate::{
    basics::{readwrite::ReadBytes, transfer::Transfer},
    object::{Construct, ConstructionTask, Deconstruct, DeconstructionTask, GlTracker, ObjectID, PipelineObject},
    pipeline::Pipeline,
    utils::{AccessType, UpdateFrequency, UsageType},
};

// An OpenGL SSBO
pub struct ShaderStorage {
    // The OpenGL name for the underlying buffer
    pub(crate) oid: u32,
    // How we access the shader storage
    pub usage: UsageType,
    // Some default data
    pub(crate) bytes: Vec<u8>,
    // The size in bytes of the underlying data
    pub(crate) byte_size: usize,
}
impl PipelineObject for ShaderStorage {
    // Reserve an ID for this shader storage
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, ObjectID::new(pipeline.shader_storages.get_next_id_increment())))
    }
    // Send this shader storage to the pipeline for construction
    fn send(self, pipeline: &Pipeline, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::ShaderStorage(Construct::<Self>(self, id))
    }
    // Create a deconstruction task
    fn pull(pipeline: &Pipeline, id: ObjectID<Self>) -> DeconstructionTask {
        DeconstructionTask::ShaderStorage(Deconstruct::<Self>(id))
    }
    // Add the shader storage to our ordered vec
    fn add(mut self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // Create the SSBO
        unsafe {
            gl::GenBuffers(1, &mut self.oid);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.oid);
            // Get the default data if we need to
            let data_ptr = if !self.bytes.is_empty() {
                self.bytes.as_ptr() as *const c_void
            } else {
                null() as *const c_void
            };
            gl::BufferData(gl::SHADER_STORAGE_BUFFER, self.byte_size as isize, data_ptr, self.usage.convert());
        }
        // Add the shader storage
        pipeline.shader_storages.insert(id.get()?, self);
        Some(())
    }
    // Remove the compute shader from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        pipeline.shader_storages.remove(id.get()?)
    }
}
impl ShaderStorage {
    // Create a new empty shader storage
    pub fn new(frequency: UpdateFrequency, access: AccessType, byte_size: usize) -> Self {
        Self {
            oid: 0,
            usage: UsageType { frequency, access },
            bytes: Vec::new(),
            byte_size,
        }
    }
    // Create a new shader storage with some default data
    // Type T must have a repr(C) layout
    pub fn new_default<T: Sized>(frequency: UpdateFrequency, access: AccessType, default: T, byte_size: usize) -> Self {
        let borrow = &default;
        let slice = unsafe { std::slice::from_raw_parts::<u8>(borrow as *const T as *const u8, byte_size) };
        Self {
            oid: 0,
            usage: UsageType { frequency, access },
            bytes: slice.to_vec(),
            byte_size,
        }
    }
    // Read some bytes from the SSBO
    pub(crate) fn read_bytes(&self, pipeline: &Pipeline, read: Transfer<ReadBytes>) -> GlTracker {
        GlTracker::new(
            move |pipeline| unsafe {
                // Bind the buffer before reading
                gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.oid);
                gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.oid);
                // If we have a range, we can use it
                let range = read.0.range;
                let bytes = if let Some(range) = range {
                    // Read using specific range
                    let offset = range.start;
                    let size = range.end - range.start;
                    // Since we use a range, make a vector that can only hold that range
                    let mut vec = vec![0; size as usize];
                    gl::GetBufferSubData(gl::SHADER_STORAGE_BUFFER, offset as isize, size as isize, vec.as_mut_ptr() as *mut c_void);
                    vec
                } else {
                    // Read the whole buffer
                    let mut vec = vec![0; self.byte_size as usize];
                    gl::GetBufferSubData(gl::SHADER_STORAGE_BUFFER, 0, self.byte_size as isize, vec.as_mut_ptr() as *mut c_void);
                    vec
                };
                // Now store the shader storage's bytes
                let mut output_bytes = read.0.bytes.lock().unwrap();
                *output_bytes = bytes;
            },
            |_| {},
            pipeline,
        )
    }
}
