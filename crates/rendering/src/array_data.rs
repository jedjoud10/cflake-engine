use std::{ffi::c_void, os::raw::c_float, ptr::null};

// A shader storage buffer object struct
#[derive(Default)]
pub struct ArrayData {
    pub buf_id: u32,
    pub len: usize,
}
impl ArrayData {
    // Create the array data using the max size that the array can possibly get to
    pub fn create_array<T: Sized>(&mut self, max_size: usize) {
        let mut ssbo: u32 = 0;
        unsafe {
            gl::GenBuffers(1, &mut ssbo);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, ssbo);
            // Size in bytes
            let size = std::mem::size_of::<T>() * max_size;
            gl::BufferData(gl::SHADER_STORAGE_BUFFER, size as isize, null(), gl::DYNAMIC_READ);
            self.len = max_size;
            self.buf_id = ssbo;
            // Unbind
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
            errors::ErrorCatcher::catch_opengl_errors().expect("Could not create ArrayData initial data!");
        }
    }
    // Create an array with preset data
    pub fn create_array_preset<T: Sized>(&mut self, vec: Vec<T>) {
        let mut ssbo: u32 = 0;
        unsafe {
            gl::GenBuffers(1, &mut ssbo);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, ssbo);
            // Size in bytes
            let size = std::mem::size_of::<T>() * vec.len();
            gl::BufferData(gl::SHADER_STORAGE_BUFFER, size as isize, vec.as_ptr() as *const c_void, gl::DYNAMIC_READ);
            self.len = vec.len();
            self.buf_id = ssbo;
            // Unbind
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
            errors::ErrorCatcher::catch_opengl_errors().expect("Could not create ArrayData initial data!");
        }
    }
    // Bind the array to the specific binding
    pub fn bind(&self, binding: u32) {
        unsafe {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, binding, self.buf_id);
        }
    }
    // Clear the data of this array data 
    pub fn clear(&mut self) {
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.buf_id);
            let val = 0.0f32;
            let ptr = &val as *const c_float as *const c_void;
            gl::ClearNamedBufferData(gl::SHADER_STORAGE_BUFFER, gl::R32F, gl::RED, gl::FLOAT, ptr);
            errors::ErrorCatcher::catch_opengl_errors().expect("Could not clear ArrayData SSBO!");
        }
    }
    // Read back the data from this array data
    pub fn read<T: Sized + Clone>(&self) -> Vec<T> {
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.buf_id);
            let ptr = gl::MapBuffer(gl::SHADER_STORAGE_BUFFER, gl::WRITE_ONLY).cast::<Vec<T>>();
            let vec = ptr.as_ref().unwrap().clone();
            // Check for corruption
            let corrupt = gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER);
            if corrupt == gl::TRUE {
                // Le bruh
                panic!("ArrayData data is corrupt!");
            } else {
                // No corruption, we're good
            }
            errors::ErrorCatcher::catch_opengl_errors().expect("Could not read back ArrayData array!");
            return vec;
        }
    }
}