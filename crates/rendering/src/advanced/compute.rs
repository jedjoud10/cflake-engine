use std::{
    ffi::c_void,
    sync::{Arc, Mutex},
};

use crate::{TextureType, object::PipelineObjectID, Texture, SharedPipeline};

// A singular compute shader sub task
pub enum ComputeShaderSubTask {
    TextureFillArray(PipelineObjectID<Texture>, usize, Arc<Mutex<Vec<u8>>>),
}

// Some compute shader tasks that we can execute after we asynchronously run a compute shader
pub struct ComputeShaderSubTasks {
    tasks: Vec<ComputeShaderSubTask>,
}

impl ComputeShaderSubTasks {
    pub fn new(tasks: Vec<ComputeShaderSubTask>) -> Self {
        Self { tasks }
    }
    // Execute the sub tasks
    pub fn run(self, pipeline: &SharedPipeline) {
        for task in self.tasks {
            match task {
                ComputeShaderSubTask::TextureFillArray(texture_id, bytecount_per_pixel, return_bytes) => {
                    let texture = if let GPUObject::Texture(x) = buf.get_gpuobject(&texture_id).unwrap() {
                        x
                    } else {
                        panic!()
                    };
                    // Get the length of the vector
                    let length: usize = match texture.ttype {
                        TextureType::Texture1D(x) => (x as usize),
                        TextureType::Texture2D(x, y) => (x as usize * y as usize),
                        TextureType::Texture3D(x, y, z) => (x as usize * y as usize * z as usize),
                        TextureType::TextureArray(_, _, _) => todo!(),
                    };
                    // Get the byte size
                    let byte_length = bytecount_per_pixel * length;

                    // Create the vector
                    let mut pixels: Vec<u8> = vec![0; byte_length];

                    let tex_type = match texture.ttype {
                        TextureType::Texture1D(_) => gl::TEXTURE_1D,
                        TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
                        TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
                        TextureType::TextureArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
                    };

                    // Actually read the pixels
                    unsafe {
                        // Bind the buffer before reading
                        gl::BindTexture(tex_type, texture.texture_id);
                        let (_internal_format, format, data_type) = texture.ifd;
                        gl::GetTexImage(tex_type, 0, format, data_type, pixels.as_mut_ptr() as *mut c_void);
                        gl::Finish();
                    }

                    // Update the vector that was given using the AtomicPtr
                    let mut new_bytes = return_bytes.as_ref().lock().unwrap();
                    *new_bytes = pixels;
                }
            }
        }
    }
}
