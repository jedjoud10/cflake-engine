use crate::engine::core::cacher::CacheManager;
use crate::engine::resources::Resource;
use crate::engine::{resources::ResourceManager};
use gl;

use std::{
    ffi::{CString},
    ptr::null,
};

use super::texture::{Texture};

// A shader that contains two sub shaders that are compiled independently
pub struct Shader {
    pub name: String,
    pub program: u32,
    pub finalized: bool,
    pub linked_subshaders_programs: Vec<u32>,
}

impl Default for Shader {
    fn default() -> Self {
        unsafe {
            Self {
                name: String::from("Undefined"),
                program: gl::CreateProgram(),
                finalized: false,
                linked_subshaders_programs: Vec::new(),
            }
        }
    }
}

impl Shader {
    // Creates a shader from multiple subshader files
    pub fn new<'a>(
        subshader_paths: Vec<&str>,
        resource_cacher: &'a mut ResourceManager,
        shader_cacher: &'a mut (CacheManager<SubShader>, CacheManager<Shader>),
    ) -> (&'a mut Self, String) {
        let mut shader = Self::default();
        // Create the shader name
        shader.name = subshader_paths.join("__");
        let name = shader.name.clone();
        // Loop through all the subshaders and link them
        for subshader_path in subshader_paths {
            // Check if we even have the subshader cached
            if shader_cacher.0.is_cached(subshader_path) {
                shader.link_subshader(shader_cacher.0.get_object(subshader_path).unwrap());
            } else {
                // It was not cached, so we need to cache it
                let resource = resource_cacher
                    .load_packed_resource(subshader_path)
                    .unwrap();
                let mut subshader = SubShader::from_resource(resource).unwrap();
                // Compile the subshader
                subshader.compile_subshader();

                // Cache it, and link it
                let _subshader = shader_cacher.0.cache_object(subshader, subshader_path);
                shader.link_subshader(shader_cacher.0.get_object(subshader_path).unwrap());
            }
        }
        // Finalize the shader and cache it
        shader.finalize_shader();
        let cached_shader_id = shader_cacher.1.cache_object(shader, &name);
        return (
            shader_cacher
                .1
                .id_get_object_mut(cached_shader_id)
                .unwrap(),
            name,
        );
    }
    // Finalizes a vert/frag shader by compiling it
    pub fn finalize_shader(&mut self) {
        unsafe {
            // Finalize the shader and stuff
            gl::LinkProgram(self.program);

            // Check for any errors
            let mut info_log_length: i32 = 0;
            let info_log_length_ptr: *mut i32 = &mut info_log_length;
            let mut result: i32 = 0;
            let result_ptr: *mut i32 = &mut result;
            gl::GetProgramiv(self.program, gl::INFO_LOG_LENGTH, info_log_length_ptr);
            gl::GetProgramiv(self.program, gl::LINK_STATUS, result_ptr);
            // Print any errors that might've happened while finalizing this shader
            if info_log_length > 0 {
                let mut log: Vec<i8> = vec![0; info_log_length as usize + 1];
                gl::GetProgramInfoLog(
                    self.program,
                    info_log_length,
                    std::ptr::null_mut::<i32>(),
                    log.as_mut_ptr(),
                );
                println!("Error while finalizing shader {}!:", self.name);
                let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
                let string = String::from_utf8(printable_log).unwrap();
                println!("Error: \n\x1b[31m{}", string);
                println!("\x1b[0m");
                panic!();
            }

            for subshader_program in self.linked_subshaders_programs.iter() {
                gl::DetachShader(self.program, *subshader_program);
            }
            self.finalized = true;
        }
    }
    // Use this shader for rendering a specific entity
    pub fn use_shader(&self) {
        // Check if the program even was finalized and ready for use
        if self.finalized {
            unsafe {
                gl::UseProgram(self.program);
            }
        }
    }
    // Link a specific subshader to this shader
    pub fn link_subshader(&mut self, subshader: &SubShader) {
        self.linked_subshaders_programs.push(subshader.program);
        unsafe {
            gl::AttachShader(self.program, subshader.program);
        }
    }
}

// Impl block for interfacing with the OpenGL shader, like setting uniforms and scuh
impl Shader {
    // Get the location of a specific uniform, using it's name
	#[allow(temporary_cstring_as_ptr)]
    pub fn get_uniform_location(&self, name: &str) -> i32 {
        unsafe {
            gl::GetUniformLocation(self.program, CString::new(name).unwrap().as_ptr())
        }
    }
    // Set a scalar uniform
    pub fn set_scalar_1_uniform(&self, name: &str, value: f32) {
        unsafe {
            gl::Uniform1f(self.get_uniform_location(name), value);
        }
    }
    // Set a scalar x2 uniform
    pub fn set_scalar_2_uniform(&self, name: &str, values: (f32, f32)) {
        unsafe {
            gl::Uniform2f(self.get_uniform_location(name), values.0, values.1);
        }
    }
    // Set a scalar x3 uniform
    pub fn set_scalar_3_uniform(&self, name: &str, values: (f32, f32, f32)) {
        unsafe {
            gl::Uniform3f(
                self.get_uniform_location(name),
                values.0,
                values.1,
                values.2,
            );
        }
    }
    // Set a matrix 4x4
    pub fn set_matrix_44_uniform(&self, name: &str, matrix: glam::Mat4) {
        unsafe {
            let ptr: *const f32 = &matrix.as_ref()[0];
            gl::UniformMatrix4fv(self.get_uniform_location(name), 1, gl::FALSE, ptr);
        }
    }
    // Set a texture
    pub fn set_texture2d(&self, name: &str, texture: &Texture, active_texture_id: u32) {
        unsafe {
			gl::ActiveTexture(active_texture_id);
			gl::BindTexture(gl::TEXTURE_2D, texture.id);
			gl::Uniform1i(
			self.get_uniform_location(name),
			active_texture_id as i32 - 33984,
			);           
        }
    }
    // Set a int
    pub fn set_int_uniform(&self, name: &str, value: i32) {
        unsafe {
            gl::Uniform1i(self.get_uniform_location(name), value);
        }
    }
}

// Sub shader type
#[derive(Debug, Clone)]
pub enum SubShaderType {
    Vertex,
    Fragment,
}

// A sub shader, could be a geometry, vertex, or fragment shader
#[derive(Clone)]
pub struct SubShader {
    pub program: u32,
    pub name: String,
    pub source: String,
    pub subshader_type: SubShaderType,
}

impl SubShader {
    // Create a subshader from a loaded subshader resource
    pub fn from_resource(resource: &Resource) -> Option<Self> {
        match resource {
            Resource::Shader(shader, shader_name) => {
                // Turn the loaded sub shader into a normal sub shader
                let subshader = Self {
                    name: shader_name.clone(),
                    program: 0,
                    source: shader.source.clone(),
                    subshader_type: shader.subshader_type.clone(),
                };
                Some(subshader)
            }
            _ => None,
        }
    }
    // Compile the current subshader's source code
    pub fn compile_subshader(&mut self) {
        let mut shader_type: u32 = 0;
        match self.subshader_type {
            SubShaderType::Vertex => shader_type = gl::VERTEX_SHADER,
            SubShaderType::Fragment => shader_type = gl::FRAGMENT_SHADER,
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
            let mut result: i32 = 0;
            let result_ptr: *mut i32 = &mut result;
            gl::GetShaderiv(self.program, gl::INFO_LOG_LENGTH, info_log_length_ptr);
            gl::GetShaderiv(self.program, gl::LINK_STATUS, result_ptr);
            // Print any errors that might've happened while compiling this subshader
            if info_log_length > 0 {
                let mut log: Vec<i8> = vec![0; info_log_length as usize + 1];
                gl::GetShaderInfoLog(
                    self.program,
                    info_log_length,
                    std::ptr::null_mut::<i32>(),
                    log.as_mut_ptr(),
                );
                println!("Error while compiling sub-shader {}!:", self.name);
                let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
                let string = String::from_utf8(printable_log).unwrap();
                println!("Error: \n\x1b[31m{}", string);
                println!("\x1b[0m");
                panic!();
            }

            println!(
                "\x1b[32mSubshader {} compiled succsessfully!\x1b[0m",
                self.name
            );
        }
    }
}
