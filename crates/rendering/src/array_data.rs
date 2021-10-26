use std::ffi::c_void;

// A shader storage buffer object struct
pub struct ArrayData {
    pub buf_id: u32
}

impl ArrayData {
    // Create the array data using a vector full of types
    pub fn create_array<T: Sized>(&mut self, arr: Vec<T>) -> bool {
        let mut ssbo: u32 = 0;
        unsafe {
            gl::GenBuffers(1, &mut ssbo);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, ssbo);
            // Size in bytes
            let size = std::mem::size_of::<T>() * arr.len();
            gl::BufferData(gl::SHADER_STORAGE_BUFFER, size as isize, arr.as_ptr() as *const c_void, gl::DYNAMIC_COPY);
            self.buf_id = ssbo;
            // Unbind
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }
        // Did we create the SSBO sucsessfully?
        return true;
    }
    // Bind the array to the specific binding
    pub fn bind(&self, binding: u32) {
        unsafe {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, binding, self.buf_id);
        }
    }
}