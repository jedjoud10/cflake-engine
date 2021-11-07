use crate::ComputeShader;
use crate::SubShader;
use crate::SubShaderType;
use crate::Texture;
use crate::TextureShaderAccessType;
use assets::Asset;
use assets::AssetManager;
use assets::Object;
use errors::RenderingError;
use gl;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::CString;
use std::rc::Rc;

// A shader that contains two sub shaders that are compiled independently
pub struct Shader {
    pub name: String,
    pub program: u32,
    pub finalized: bool,
    pub linked_subshaders_programs: Vec<(SubShaderType, u32)>,
    pub additional_shader: AdditionalShader,
    pub additional_shader_sources: Vec<String>,
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
                additional_shader_sources: Vec::new(),
            }
        }
    }
}

// A shader is an asset object, while a subshader is an asset
impl Object for Shader {
    fn get_unique_object_name(&self, local_path: &str) -> String {
        self.name.to_string()
    }
}

impl Shader {
    // Load the files that need to be included for this specific shader and return the included lines
    fn load_includes<'a>(&self, lines: Vec<String>, asset_manager: &'a AssetManager, included_paths: &mut HashSet<String>) -> Vec<String> {
        let mut included_lines: Vec<String> = Vec::new();
        for line in lines.iter() {
            // Check if this is an include statement
            if line.starts_with("#include ") {
                // Get the local path of the include file
                let local_path = line.split("#include ").collect::<Vec<&str>>()[1].replace(r#"""#, "");
                let local_path = local_path.trim_start();
                if !included_paths.contains(&local_path.to_string()) {
                    // Load the function shader text
                    included_paths.insert(local_path.to_string());
                    let text = asset_manager.asset_cacher.load_text(local_path).unwrap();
                    let new_lines = text.lines().map(|x| x.to_string()).collect::<Vec<String>>();
                    included_lines.extend(new_lines);
                }
            }
            // Custom shader sources
            if self.additional_shader_sources.len() > 0 {
                if line.starts_with("#include_custom") {
                    // Get the source
                    let c = line.split("#include_custom ").collect::<Vec<&str>>()[1];
                    let source_id = c[2..(c.len() - 2)].parse::<usize>().unwrap();
                    let source = self.additional_shader_sources.get(source_id).unwrap();
                    let lines = source.lines().map(|x| x.to_string()).collect::<Vec<String>>();
                    included_lines.extend(lines);
                }
            }
        }
        // Return the included lines and the original lines that are without the include statement
        return included_lines;
    }
    // Create a new empty shader
    pub fn new() -> Self {
        Self::default()
    }
    // Set an additional shader for this shader before we finalize it
    pub fn set_additional_shader(mut self, additional_shader: AdditionalShader) -> Self {
        self.additional_shader = additional_shader;
        self
    }
    // Set an additional shader source that can be loaded at runtime
    pub fn set_additional_shader_sources(mut self, additional_shader_sources: Vec<&str>) -> Self {
        self.additional_shader_sources = additional_shader_sources.into_iter().map(|x| x.to_string()).collect();
        self
    }
    // Creates a shader from multiple subshader files
    pub fn load_shader<'a>(mut self, subshader_paths: Vec<&str>, asset_manager: &'a mut AssetManager) -> Option<Self> {
        // Create the shader name
        self.name = subshader_paths.join("__");
        let mut included_paths: HashSet<String> = HashSet::new();
        // Loop through all the subshaders and link them
        for subshader_path in subshader_paths {
            // Check if we even have the subshader cached
            if asset_manager.object_cacher.cached(subshader_path) {
                let rc_subshader = SubShader::object_load_o(subshader_path, &asset_manager.object_cacher);
                let subshader = rc_subshader.as_ref();
                self.link_subshader(subshader);
            } else {
                // It was not cached, so we need to cache it
                let mut subshader = SubShader::asset_load(asset_manager.asset_cacher.load_md(subshader_path).unwrap())?;

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
                    let orignal_local_included_lines = self.load_includes(lines.clone(), asset_manager, &mut included_paths);
                    // Extend from the start of the vector
                    let mut local_included_lines = orignal_local_included_lines.clone();
                    local_included_lines.extend(included_lines);
                    included_lines = local_included_lines.clone();

                    shader_sources_to_evalute.remove(0);
                    // Check if the added included lines aren't empty
                    if !orignal_local_included_lines.is_empty() {
                        shader_sources_to_evalute.push(orignal_local_included_lines);
                    }
                }
                // Set the shader source for this shader
                let extend_shader_source = included_lines.join("\n");

                // Remove the version directive from the original subshader source
                let og_shader_source = subshader.source.split(&version_directive).nth(1)?;
                subshader.source = format!("{}\n{}\n{}", version_directive, extend_shader_source, og_shader_source);
                // Gotta filter out the include messages
                subshader.source = subshader
                    .source
                    .lines()
                    .filter(|x| {
                        let s = x.to_string();
                        let s = s.trim();
                        !s.starts_with("#include") && !s.starts_with("#include_custom")
                    })
                    .collect::<Vec<&str>>()
                    .join("\n");
                //println!("{}", subshader.source);
                // Compile the subshader
                subshader.compile_subshader();

                // Cache it, and link it
                let rc_subshader = asset_manager.object_cacher.cache(subshader_path, subshader).ok()?;
                let _subshader = rc_subshader.as_ref();
                self.link_subshader(_subshader);
                // Unload the resource since we just cached the shader
                //resource_manager.unload_resouce(subshader_path);
            }
        }
        // Finalize the shader and cache it
        self.finalize_shader();
        return Some(self);
    }
    // Cache this shader
    pub fn cache<'a>(self, asset_manager: &'a mut AssetManager) -> Rc<Self> {
        let name = self.name.clone();
        let shader = asset_manager.object_cacher.cache(&name, self).unwrap();
        return shader;
    }
    // Finalizes a vert/frag shader by compiling it
    fn finalize_shader(&mut self) {
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

// Some default uniforms that we will set
#[derive(Clone)]
pub enum Uniform {
    // Singles
    F32(f32),
    I32(i32),
    // Vectors
    Vec2F32(veclib::Vector2<f32>),
    Vec3F32(veclib::Vector3<f32>),
    Vec4F32(veclib::Vector4<f32>),
    Vec2I32(veclib::Vector2<i32>),
    Vec3I32(veclib::Vector3<i32>),
    Vec4I32(veclib::Vector4<i32>),
    Mat44F32(veclib::Matrix4x4<f32>),
    // Others
    Texture2D(Rc<Texture>, u32),
    Texture3D(Rc<Texture>, u32),
    Texture2DArray(Rc<Texture>, u32),
}

// Impl block for interfacing with the OpenGL shader, like setting uniforms and scuh
impl Shader {
    // Get the location of a specific uniform, using it's name
    #[allow(temporary_cstring_as_ptr)]
    pub fn get_uniform_location(&self, name: &str) -> Result<i32, RenderingError> {
        unsafe {
            let x = gl::GetUniformLocation(self.program, CString::new(name).unwrap().as_ptr());
            return Ok(x);
        }
    }
    // Set a f32 uniform
    pub fn set_f32(&self, name: &str, value: &f32) {
        let u = self.get_uniform_location(name).unwrap();
        unsafe {
            gl::Uniform1f(u, *value);
        }
    }
    // Set a vec2 f32 uniform
    pub fn set_vec2f32(&self, name: &str, vec: &veclib::Vector2<f32>) {
        let u = self.get_uniform_location(name).unwrap();
        unsafe {
            gl::Uniform2f(u, vec[0], vec[1]);
        }
    }
    // Set a vec3 f32 uniform
    pub fn set_vec3f32(&self, name: &str, vec: &veclib::Vector3<f32>) {
        let u = self.get_uniform_location(name).unwrap();
        unsafe {
            gl::Uniform3f(u, vec[0], vec[1], vec[2]);
        }
    }
    // Set a vec3 f32 array uniform
    pub fn set_vec3f32_array(&self, name: &str, vec: &[veclib::Vector3<f32>]) {
        let u = self.get_uniform_location(name).unwrap();
        unsafe {
            let ptr: *const f32 = &vec[0].x;
            gl::Uniform3fv(u, vec.len() as i32, ptr);
        }
    }
    // Set a vec4 f32 uniform
    pub fn set_vec4f32(&self, name: &str, vec: &veclib::Vector4<f32>) {
        let u = self.get_uniform_location(name).unwrap();
        unsafe {
            gl::Uniform4f(u, vec[0], vec[1], vec[2], vec[3]);
        }
    }
    // Set a matrix 4x4 f32
    pub fn set_mat44(&self, name: &str, matrix: &veclib::Matrix4x4<f32>) {
        let u = self.get_uniform_location(name).unwrap();
        unsafe {
            let ptr: *const f32 = &matrix[0];
            gl::UniformMatrix4fv(u, 1, gl::FALSE, ptr);
        }
    }
    // Set a 2D texture
    pub fn set_t2d(&self, name: &str, texture: &Texture, active_texture_id: u32) {
        let u = self.get_uniform_location(name).unwrap();
        unsafe {
            if u != -1 {
                gl::ActiveTexture(active_texture_id + 33984);
                gl::BindTexture(gl::TEXTURE_2D, texture.id);
                gl::Uniform1i(u, active_texture_id as i32);
            }
        }
    }
    // Set a 3D texture
    pub fn set_t3d(&self, name: &str, texture: &Texture, active_texture_id: u32) {
        let u = self.get_uniform_location(name).unwrap();
        unsafe {
            if u != -1 {
                gl::ActiveTexture(active_texture_id + 33984);
                gl::BindTexture(gl::TEXTURE_3D, texture.id);
                gl::Uniform1i(u, active_texture_id as i32);
            }
        }
    }
    // Set a 2D image
    pub fn set_i2d(&self, name: &str, texture: &Texture, access_type: TextureShaderAccessType) {
        let u = self.get_uniform_location(name).unwrap();
        unsafe {
            // Converstion from wrapper to actual opengl values
            let new_access_type: u32;
            match access_type {
                TextureShaderAccessType::ReadOnly => new_access_type = gl::READ_ONLY,
                TextureShaderAccessType::WriteOnly => new_access_type = gl::WRITE_ONLY,
                TextureShaderAccessType::ReadWrite => new_access_type = gl::READ_WRITE,
            };
            let unit = u as u32;
            gl::BindTexture(gl::TEXTURE_2D, texture.id);
            gl::BindImageTexture(unit, texture.id, 0, gl::FALSE, 0, new_access_type, texture.internal_format);
        }
    }
    // Set a 3D image
    pub fn set_i3d(&self, name: &str, texture: &Texture, access_type: TextureShaderAccessType) {
        let u = self.get_uniform_location(name).unwrap();
        unsafe {
            // Converstion from wrapper to actual opengl values
            let new_access_type: u32;
            match access_type {
                TextureShaderAccessType::ReadOnly => new_access_type = gl::READ_ONLY,
                TextureShaderAccessType::WriteOnly => new_access_type = gl::WRITE_ONLY,
                TextureShaderAccessType::ReadWrite => new_access_type = gl::READ_WRITE,
            };
            let unit = u as u32;
            gl::BindTexture(gl::TEXTURE_3D, texture.id);
            gl::BindImageTexture(unit, texture.id, 0, gl::FALSE, 0, new_access_type, texture.internal_format);
        }
    }
    // Set a i32
    pub fn set_i32(&self, name: &str, value: &i32) {
        let u = self.get_uniform_location(name).unwrap();
        unsafe {
            gl::Uniform1i(u, *value);
        }
    }
    // Set a vec2 i32 uniform
    pub fn set_vec2i32(&self, name: &str, vec: &veclib::Vector2<i32>) {
        let u = self.get_uniform_location(name).unwrap();
        unsafe {
            gl::Uniform2i(u, vec[0], vec[1]);
        }
    }
    // Set a vec3 i32 uniform
    pub fn set_vec3i32(&self, name: &str, vec: &veclib::Vector3<i32>) {
        let u = self.get_uniform_location(name).unwrap();
        unsafe {
            gl::Uniform3i(u, vec[0], vec[1], vec[2]);
        }
    }
    // Set a vec4 i32 uniform
    pub fn set_vec4i32(&self, name: &str, vec: &veclib::Vector4<i32>) {
        let u = self.get_uniform_location(name).unwrap();
        unsafe {
            gl::Uniform4i(u, vec[0], vec[1], vec[2], vec[3]);
        }
    }
    // Set a texture2d array
    pub fn set_t2da(&self, name: &str, texture: &Texture, active_texture_id: u32) {
        let u = self.get_uniform_location(name).unwrap();
        unsafe {
            if u != 1 {
                gl::ActiveTexture(active_texture_id + 33984);
                gl::BindTexture(gl::TEXTURE_2D_ARRAY, texture.id);
                gl::Uniform1i(u, active_texture_id as i32);
            }
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
