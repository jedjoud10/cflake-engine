use gl::types::GLuint;
use std::{ffi::c_void, ptr::null, mem::size_of};

use crate::{
    basics::{
        buffer_operation::BufferOperation,
        shader::{
            info::{QueryParameter, QueryResource::ShaderStorageBlock, Resource, ShaderInfoQuerySettings},
            query_shader_info,
        },
    },
    pipeline::Pipeline,
    utils::{UsageType, AccessType, UpdateFrequency}, object::PipelineCollectionElement,
};

use super::raw::dynamic_buffer::DynamicRawBuffer;

// An OpenGL SSBO
pub struct ShaderStorage<T> {
    // Backed by a dynamic raw buffer
    storage: DynamicRawBuffer<T>
}

// Getters and mut getters
impl<T> ShaderStorage<T> {
    pub fn storage(&self) -> &DynamicRawBuffer<T> { &self.storage }
    pub fn storage_mut(&mut self) -> &mut DynamicRawBuffer<T> { &mut self.storage }
}

impl<T> ShaderStorage<T> {
    // Create a new empty shader storage
    pub fn new(usage: UsageType, _pipeline: &Pipeline) -> Self {
        Self {
            storage: DynamicRawBuffer::<T>::new(gl::SHADER_STORAGE_BUFFER, UsageType::new(AccessType::Draw, UpdateFrequency::Dynamic), _pipeline),
        }
    }
    /*
    // Read/Write some bytes from/to the SSBO
    pub(crate) fn buffer_operation(&mut self, op: BufferOperation) -> GlTracker {
        match op {
            BufferOperation::Write(mut write) => {
                GlTracker::fake(|| unsafe {
                    // Bind the buffer before writing
                    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.oid);
                    write.bytes.shrink_to_fit();
                    // If the given data contains more bytes than what we can handle, we must re-allocate the buffer and increase it's size
                    if write.bytes.len() > self.byte_size {
                        if self.dynamic {
                            // Reallocate
                            gl::BufferData(
                                gl::SHADER_STORAGE_BUFFER,
                                write.bytes.len() as isize,
                                write.bytes.as_ptr() as *const c_void,
                                self.usage.convert(),
                            );
                            self.byte_size = write.bytes.len();
                            println!("Writing to SSBO with {} bytes", write.bytes.len());
                        } else {
                            panic!("Buffer is not dynamic, cannot reallocate!");
                        }
                    } else {
                        // We have enough bytes allocated already
                        gl::BufferSubData(gl::SHADER_STORAGE_BUFFER, 0, write.bytes.len() as isize, write.bytes.as_ptr() as *const c_void);
                    }
                })
            }
            BufferOperation::Read(read) => {
                GlTracker::new(|| unsafe {
                    // Bind the buffer before reading
                    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.oid);
                    // Read the whole buffer
                    let mut vec = vec![0u8; self.byte_size as usize];
                    gl::GetBufferSubData(gl::SHADER_STORAGE_BUFFER, 0, self.byte_size as isize, vec.as_mut_ptr() as *mut c_void);
                    // Now store the shader storage's bytes
                    let mut output_bytes = read.bytes.lock();
                    *output_bytes = vec;
                })
            }
        }
    }
    */
}
