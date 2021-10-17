use crate::texture_3d::Texture3D;
use crate::ComputeShader;
use crate::SubShader;
use crate::SubShaderType;
use crate::TextureShaderAccessType;

use super::Texture2D;
use gl;
use others::CacheManager;
use resources::ResourceManager;
use std::{ffi::CString};

// A shader that contains two sub shaders that are compiled independently
pub struct Shader {
    pub name: String,
    pub program: u32,
    pub finalized: bool,
    pub linked_subshaders_programs: Vec<(SubShaderType, u32)>,
    pub additional_shader: AdditionalShader,
}

impl Default for Shader {
    fn default() -> Self {
        unsafe {
            Self {
                name: String::from("Undefined"),
                program: gl::CreateProgram(),
                finalized: false,
                linked_subshaders_programs: Vec::new(),
                additional_shader: AdditionalShader::None,
            }
        }
    }
}

impl Shader {
    // Load the files that need to be included for this specific shader and return the included lines
    fn load_includes<'a>(lines: Vec<String>, resource_manager: &'a mut ResourceManager) -> Vec<String> {
        let mut included_lines: Vec<String> = Vec::new();
        for line in lines {
            // Check if this is an include statement
            if line.starts_with("#include") {
                // Get the local path of the include file
                let local_path = line.split("#include").collect::<Vec<&str>>()[1].replace(r#"""#, "");
                let local_path = local_path.trim_start();
                // Load the function shader text
                let text = resource_manager.load_lines_packed_resource(&local_path, 1).unwrap();
                let new_lines = text.lines().map(|x| x.to_string()).collect::<Vec<String>>();
                included_lines.extend(new_lines);
            }
        }
        // Return the included lines and the original lines that are without the include statement
        return included_lines;
    }
    // Creates a shader from multiple subshader files
    pub fn new<'a>(
        subshader_paths: Vec<&str>,
        resource_manager: &'a mut ResourceManager,
        shader_cacher: &'a mut (CacheManager<SubShader>, CacheManager<Shader>),
        additional_shader: Option<AdditionalShader>,
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
                let resource = resource_manager.load_packed_resource(subshader_path).unwrap();
                let mut subshader = SubShader::from_resource(resource).unwrap();

                // Recursively load the shader includes
                let lines = subshader.source.lines().collect::<Vec<&str>>();
                let lines = lines.clone().iter().map(|x| x.to_string()).collect::<Vec<String>>();
                let mut version_directive: String = String::new();
                // Save the version directive
                for (_, line) in lines.iter().enumerate() {
                    if line.starts_with("#version") {
                        version_directive = line.clone();
                        break;
                    }
                }
                // The list of the shaders that need to be evaluated
                let mut shader_sources_to_evalute: Vec<Vec<String>> = vec![lines];
                // The final included lines
                let mut included_lines: Vec<String> = Vec::new();
                while shader_sources_to_evalute.len() > 0 {
                    // Get the lines
                    let lines = shader_sources_to_evalute[0].clone();
                    // Recursively load the includes
                    let orignal_local_included_lines = Self::load_includes(lines.clone(), resource_manager);
                    // Extend from the start of the vector
                    let mut local_indluded_lines = orignal_local_included_lines.clone();
                    local_indluded_lines.extend(included_lines);
                    included_lines = local_indluded_lines.clone();

                    shader_sources_to_evalute.remove(0);
                    // Check if the added included lines aren't empty
                    if !orignal_local_included_lines.is_empty() {
                        shader_sources_to_evalute.push(orignal_local_included_lines);
                    }
                }
                // Set the shader source for this shader
                let extend_shader_source = included_lines.join("\n");

                // Remove the version directive from the original subshader source
                let og_shader_source = subshader.source.split(&version_directive).nth(1).unwrap();
                subshader.source = format!("{}\n{}\n{}", version_directive, extend_shader_source, og_shader_source);
                // Gotta filter out the include messages
                subshader.source = subshader.source.lines().filter(|x| !x.starts_with("#include")).collect::<Vec<&str>>().join("\n");
                //println!("{}", subshader.source);
                // Compile the subshader
                subshader.compile_subshader();

                // Cache it, and link it
                let _subshader = shader_cacher.0.cache_object(subshader, subshader_path);
                shader.link_subshader(shader_cacher.0.get_object(subshader_path).unwrap());
            }
        }
        // Set the additional shader
        shader.additional_shader = additional_shader.unwrap_or(AdditionalShader::None);
        // Finalize the shader and cache it
        shader.finalize_shader();
        let cached_shader_id = shader_cacher.1.cache_object(shader, &name);
        return (shader_cacher.1.id_get_object_mut(cached_shader_id).unwrap(), name);
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
                gl::GetProgramInfoLog(self.program, info_log_length, std::ptr::null_mut::<i32>(), log.as_mut_ptr());
                println!("Error while finalizing shader {}!:", self.name);
                let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
                let string = String::from_utf8(printable_log).unwrap();
                println!("Error: \n\x1b[31m{}", string);
                println!("\x1b[0m");
                panic!();
            }

            for subshader_program in self.linked_subshaders_programs.iter() {
                gl::DetachShader(self.program, subshader_program.1);
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
            match errors::ErrorCatcher::catch_opengl_errors() {
                Some(_x) => {}
                None => {
                    println!("{:?}", self.name);
                }
            }
        }
    }
    // Link a specific subshader to this shader
    pub fn link_subshader(&mut self, subshader: &SubShader) {
        self.linked_subshaders_programs.push((subshader.subshader_type, subshader.program));
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
        unsafe { gl::GetUniformLocation(self.program, CString::new(name).unwrap().as_ptr()) }
    }
    // Set a f32 uniform
    pub fn set_f32(&self, name: &str, value: &f32) {
        let u = self.get_uniform_location(name);
        if u == -1 {
            return; /* Return early if the uniform location is invalid */
        }
        unsafe {
            gl::Uniform1f(u, *value);
        }
    }
    // Set a vec2 f32 uniform
    pub fn set_vec2f32(&self, name: &str, vec: &veclib::Vector2<f32>) {
        let u = self.get_uniform_location(name);
        if u == -1 {
            return; /* Return early if the uniform location is invalid */
        }
        unsafe {
            gl::Uniform2f(u, vec[0], vec[1]);
        }
    }
    // Set a vec3 f32 uniform
    pub fn set_vec3f32(&self, name: &str, vec: &veclib::Vector3<f32>) {
        let u = self.get_uniform_location(name);
        if u == -1 {
            return; /* Return early if the uniform location is invalid */
        }
        unsafe {
            gl::Uniform3f(u, vec[0], vec[1], vec[2]);
        }
    }
    // Set a vec4 f32 uniform
    pub fn set_vec4f32(&self, name: &str, vec: &veclib::Vector4<f32>) {
        let u = self.get_uniform_location(name);
        if u == -1 {
            return; /* Return early if the uniform location is invalid */
        }
        unsafe {
            gl::Uniform4f(u, vec[0], vec[1], vec[2], vec[3]);
        }
    }
    // Set a matrix 4x4 f32
    pub fn set_mat44(&self, name: &str, matrix: &veclib::Matrix4x4<f32>) {
        let u = self.get_uniform_location(name);
        if u == -1 {
            return; /* Return early if the uniform location is invalid */
        }
        unsafe {
            let ptr: *const f32 = &matrix[0][0];
            gl::UniformMatrix4fv(u, 1, gl::FALSE, ptr);
        }
    }
    // Set a 2D texture
    pub fn set_t2d(&self, name: &str, texture: &Texture2D, active_texture_id: u32) {
        let u = self.get_uniform_location(name);
        if u == -1 {
            return; /* Return early if the uniform location is invalid */
        }
        unsafe {
            gl::ActiveTexture(active_texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture.internal_texture.id);
            gl::Uniform1i(u, active_texture_id as i32 - 33984);
        }
    }
    // Set a 3D texture
    pub fn set_t3d(&self, name: &str, texture: &Texture3D, active_texture_id: u32) {
        let u = self.get_uniform_location(name);
        if u == -1 {
            return; /* Return early if the uniform location is invalid */
        }
        unsafe {
            gl::ActiveTexture(active_texture_id);
            gl::BindTexture(gl::TEXTURE_3D, texture.internal_texture.id);
            gl::Uniform1i(u, active_texture_id as i32 - 33984);
        }
    }
    // Set a 2D image
    pub fn set_i2d(&self, name: &str, texture: &Texture2D, access_type: TextureShaderAccessType) {
        let u = self.get_uniform_location(name);
        if u == -1 {
            return; /* Return early if the uniform location is invalid */
        }
        unsafe {
            // Converstion from wrapper to actual opengl values
            let new_access_type: u32;
            match access_type {
                TextureShaderAccessType::ReadOnly => new_access_type = gl::READ_ONLY,
                TextureShaderAccessType::WriteOnly => new_access_type = gl::WRITE_ONLY,
                TextureShaderAccessType::ReadWrite => new_access_type = gl::READ_WRITE,
            };
            let unit = u as u32;
            gl::BindTexture(gl::TEXTURE_2D, texture.internal_texture.id);
            gl::BindImageTexture(
                unit,
                texture.internal_texture.id,
                0,
                gl::FALSE,
                0,
                new_access_type,
                texture.internal_texture.internal_format,
            );
        }
    }
    // Set a 3D image
    pub fn set_i3d(&self, name: &str, texture: &Texture3D, access_type: TextureShaderAccessType) {
        let u = self.get_uniform_location(name);
        if u == -1 {
            return; /* Return early if the uniform location is invalid */
        }
        unsafe {
            // Converstion from wrapper to actual opengl values
            let new_access_type: u32;
            match access_type {
                TextureShaderAccessType::ReadOnly => new_access_type = gl::READ_ONLY,
                TextureShaderAccessType::WriteOnly => new_access_type = gl::WRITE_ONLY,
                TextureShaderAccessType::ReadWrite => new_access_type = gl::READ_WRITE,
            };
            let unit = u as u32;
            gl::BindTexture(gl::TEXTURE_3D, texture.internal_texture.id);
            gl::BindImageTexture(
                unit,
                texture.internal_texture.id,
                0,
                gl::FALSE,
                0,
                new_access_type,
                texture.internal_texture.internal_format,
            );
        }
    }
    // Set a i32
    pub fn set_i32(&self, name: &str, value: &i32) {
        let u = self.get_uniform_location(name);
        if u == -1 {
            return; /* Return early if the uniform location is invalid */
        }
        unsafe {
            gl::Uniform1i(u, *value);
        }
    }
    // Set a vec2 i32 uniform
    pub fn set_vec2i32(&self, name: &str, vec: &veclib::Vector2<i32>) {
        let u = self.get_uniform_location(name);
        if u == -1 {
            return; /* Return early if the uniform location is invalid */
        }
        unsafe {
            gl::Uniform2i(u, vec[0], vec[1]);
        }
    }
    // Set a vec3 i32 uniform
    pub fn set_vec3i32(&self, name: &str, vec: &veclib::Vector3<i32>) {
        let u = self.get_uniform_location(name);
        if u == -1 {
            return; /* Return early if the uniform location is invalid */
        }
        unsafe {
            gl::Uniform3i(u, vec[0], vec[1], vec[2]);
        }
    }
    // Set a vec4 i32 uniform
    pub fn set_vec4i32(&self, name: &str, vec: &veclib::Vector4<i32>) {
        let u = self.get_uniform_location(name);
        if u == -1 {
            return; /* Return early if the uniform location is invalid */
        }
        unsafe {
            gl::Uniform4i(u, vec[0], vec[1], vec[2], vec[3]);
        }
    }
}

// Each shader can be instanced
impl others::Instance for Shader {
    fn set_name(&mut self, string: String) {
        self.name = string
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
}

// Additional shaders
pub enum AdditionalShader {
    None,
    Compute(ComputeShader),
}
