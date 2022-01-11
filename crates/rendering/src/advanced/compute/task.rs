use std::{sync::{Arc, Mutex}, ffi::c_void};
use crate::{object::ObjectID, Texture, Pipeline, TextureType};

// Some task that we will execute after we run the compute shader
pub enum ComputeShaderTask {
    FillTexture(ObjectID<Texture>, usize, Arc<Mutex<Vec<u8>>>)
}

impl ComputeShaderTask {
    // Execute a compute shader task
    pub fn execute(self, pipeline: &Pipeline) {
        match self {
            ComputeShaderTask::FillTexture(id, bytecount_per_pixel, bytes) => {
                let texture = pipeline.get_texture(id).unwrap();
                // Get the length of the vector
                let length: usize = match texture.ttype {
                    TextureType::Texture1D(x) => (x as usize),
                    TextureType::Texture2D(x, y) => (x as usize * y as usize),
                    TextureType::Texture3D(x, y, z) => (x as usize * y as usize * z as usize),
                    TextureType::Texture2DArray(_, _, _) => todo!(),
                };
                // Get the byte size
                let byte_length = bytecount_per_pixel * length;

                // Create the vector
                let mut pixels: Vec<u8> = vec![0; byte_length];

                // Actually read the pixels
                unsafe {
                    // Bind the buffer before reading
                    gl::BindTexture(texture.target, texture.oid);
                    let (_internal_format, format, data_type) = texture.ifd;
                    gl::GetTexImage(texture.target, 0, format, data_type, pixels.as_mut_ptr() as *mut c_void);
                    gl::Finish();
                }

                // Update the vector that was given using the AtomicPtr
                let mut new_bytes = bytes.as_ref().lock().unwrap();
                *new_bytes = pixels;
            },
        }
    }
}