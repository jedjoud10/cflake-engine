use std::{ffi::c_void, ptr::null};

// A shader storage buffer object struct
#[derive(Default)]
pub struct ArrayData {
    pub buf_id: u32,
    pub byte_size: usize,
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
            self.byte_size = size;
            self.buf_id = ssbo;
            // Unbind
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
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
            gl::NamedBufferStorage(ssbo, size as isize, vec.as_ptr() as *const c_void, gl::MAP_WRITE_BIT | gl::MAP_READ_BIT);
            self.byte_size = vec.len();
            self.buf_id = ssbo;
        }
    }
    // Bind the array to the specific binding
    pub fn bind(&self, binding: u32) {
        unsafe {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, binding, self.buf_id);
        }
    }
    // Read back the data from this array data
    pub fn read<T: Sized + Clone>(&self) -> Vec<T> {
        unsafe {
            let ptr = gl::MapNamedBufferRange(self.buf_id, 0, self.byte_size as isize, gl::MAP_READ_BIT).cast::<Vec<T>>();
            if ptr.as_ref().is_none() {
                // Shit
                panic!();
            }
            let vec = ptr.as_ref().unwrap().clone();
            // Check for corruption
            let corrupt = gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER);
            if corrupt == gl::TRUE {
                // Le bruh
                panic!("ArrayData data is corrupt!");
            } else {
                // No corruption, we're good
            }
            vec
        }
    }
}
