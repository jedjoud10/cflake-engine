use getset::{Getters, CopyGetters};
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
    utils::UsageType, object::PipelineCollectionElement,
};

// An OpenGL SSBO
#[derive(CopyGetters)]
pub struct ShaderStorage {
    // The OpenGL name for the underlying buffer
    #[getset(get_copy = "pub(crate)")]
    buffer: GLuint,
    // How we access the shader storage
    #[getset(get_copy = "pub(crate)")]
    usage: UsageType,
    // The size in bytes of the underlying data
    #[getset(get_copy = "pub(crate)")]
    byte_size: usize,
}

impl PipelineCollectionElement for ShaderStorage {
    fn added(&mut self, collection: &mut crate::pipeline::PipelineCollection<Self>, handle: crate::pipeline::Handle<Self>) {
        // Create the SSBO
        unsafe {
            gl::GenBuffers(1, &mut self.buffer);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.buffer);
            gl::BufferData(gl::SHADER_STORAGE_BUFFER, self.byte_size as isize, null(), self.usage.convert());
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }
    }

    fn disposed(self) {
        todo!()
    }
}



impl ShaderStorage {
    // Create a new empty shader storage
    pub fn new<T: Sized>(usage: UsageType) -> Self {
        Self {
            buffer: 0,
            usage,
            byte_size: size_of::<T>(),
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
