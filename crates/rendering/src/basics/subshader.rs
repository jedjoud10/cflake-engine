use std::{ffi::CString, ptr::null};



// Sub shader type
#[derive(Debug, Copy, Clone)]
pub enum SubShaderType {
    Vertex,
    Fragment,
    Compute,
}

impl Default for SubShaderType {
    fn default() -> Self {
        Self::Vertex
    }
}

// A sub shader, could be a geometry, vertex, or fragment shader
#[derive(Clone, Default)]
pub struct SubShader {
    pub program: u32,
    pub name: String,
    pub source: String,
    pub subshader_type: SubShaderType,
}

impl SubShader {
    // Compile the current subshader's source code
    pub fn compile_subshader(&mut self) {
        let shader_type: u32;
        match self.subshader_type {
            SubShaderType::Vertex => shader_type = gl::VERTEX_SHADER,
            SubShaderType::Fragment => shader_type = gl::FRAGMENT_SHADER,
            SubShaderType::Compute => shader_type = gl::COMPUTE_SHADER,
        }
        unsafe {
            self.program = gl::CreateShader(shader_type);
            // Compile the shader
            let cstring = CString::new(self.source.clone()).unwrap();
            let shader_source: *const i8 = cstring.as_ptr();
            gl::ShaderSource(self.program, 1, &shader_source, null());
            gl::CompileShader(self.program);
            // Check for any errors
            let mut info_log_length: i32 = 0;
            let info_log_length_ptr: *mut i32 = &mut info_log_length;
            gl::GetShaderiv(self.program, gl::INFO_LOG_LENGTH, info_log_length_ptr);
            // Print any errors that might've happened while compiling this subshader
            if info_log_length > 0 {
                let mut log: Vec<i8> = vec![0; info_log_length as usize + 1];
                gl::GetShaderInfoLog(self.program, info_log_length, std::ptr::null_mut::<i32>(), log.as_mut_ptr());
                println!("Error while compiling sub-shader {}!:", self.name);
                let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
                let string = String::from_utf8(printable_log).unwrap();

                println!("Error: \n\x1b[31m{}", string);
                println!("\x1b[0m");
                println!("{}", self.source);
                panic!();
            }

            println!("\x1b[32mSubshader {} compiled succsessfully!\x1b[0m", self.name);
        }
    }
}
