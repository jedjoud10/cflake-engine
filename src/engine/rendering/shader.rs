use crate::engine::resources::Resource;
use crate::engine::{core::world::World, resources::ResourceManager};
use gl;
use std::{
	collections::HashMap,
	ffi::{c_void, CString},
	mem::size_of,
	ptr::null,
};

// Shader manager
#[derive(Default)]
pub struct ShaderManager {
	pub shaders: HashMap<String, Shader>,
	pub subshaders: HashMap<String, SubShader>,
}

// Struct holding the names of the default shaders
#[derive(Default)]
pub struct ShaderDefaults {
	pub default_shader_name: String,
}

impl ShaderManager {
	// Caches a specific subshader, if it already exists then give back a reference
	pub fn cache_subshader(&mut self, subshader: SubShader) -> Option<&mut SubShader> {
		if !self.subshaders.contains_key(&subshader.name) {
			// Cache the shader for later use
			let name_clone = subshader.name.clone();
			self.subshaders.insert(name_clone.clone(), subshader);
			return self.subshaders.get_mut(&name_clone);
		} else {
			return self.subshaders.get_mut(&subshader.name);
		}
	}
	// Cached a specific shader (An actual runnable shader with uniforms and all)
	pub fn cache_shader(&mut self, shader: Shader) -> Option<&mut Shader> {
		if !self.shaders.contains_key(&shader.name) {
			// Cache the shader for later use
			let name_clone = shader.name.clone();
			self.shaders.insert(name_clone.clone(), shader);
			return self.shaders.get_mut(&name_clone);
		} else {
			panic!("Cannot cache the same shader twice!");
			return None;
		}
	}
	// Gets a specific subshader from cache
	pub fn get_subshader(&mut self, subshader_name: &String) -> Option<&mut SubShader> {
		// Make sure it exists
		if self.subshaders.contains_key(subshader_name) {
			return self.subshaders.get_mut(subshader_name);
		} else {
			return None;
		}
	}
	// Gets a specific shader from cache
	pub fn get_shader(&self, shader_name: &String) -> Option<&Shader> {
		// Make sure it exists
		if self.shaders.contains_key(shader_name) {
			return self.shaders.get(shader_name);
		} else {
			return None;
		}
	}
}

impl ShaderDefaults {
	// Load all the default shaders
	pub fn load_default_shaders(
		&mut self,
		resource_manager: &mut ResourceManager,
		shader_manager: &mut ShaderManager,
	) {
		// Load the default shader
		self.default_shader_name = {
			let default_shader = Shader::from_vr_fr_subshader_files(
				"default.vrsh.glsl",
				"default.frsh.glsl",
				resource_manager,
				shader_manager,
			);
			default_shader.name.clone()
		};
	}
}

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
	// Creates a shader from a vertex subshader file and a fragment subshader file
	pub fn from_vr_fr_subshader_files<'a>(
		vertex_file: &str,
		fragment_file: &str,
		resource_manager: &'a mut ResourceManager,
		shader_manager: &'a mut ShaderManager,
	) -> &'a mut Self {
		let mut shader = Self::default();
		shader.name = format!("{}_{}", vertex_file, fragment_file);
		{
			{
				let default_vert_subshader_resource = resource_manager
					.load_packed_resource(vertex_file, "shaders\\")
					.unwrap();
				// Link the vertex and fragment shaders
				let mut vert_subshader =
					SubShader::from_resource(default_vert_subshader_resource).unwrap();
				// Compile the subshader
				vert_subshader.compile_subshader();
				// Cache it, and link it
				let vert_subshader = shader_manager.cache_subshader(vert_subshader).unwrap();
				shader.link_subshader(&vert_subshader);
			}
			{
				let default_frag_subshader_resource = resource_manager
					.load_packed_resource(fragment_file, "shaders\\")
					.unwrap();
				// Link the vertex and fragment shaders
				let mut frag_subshader =
					SubShader::from_resource(default_frag_subshader_resource).unwrap();
				// Compile the subshader
				frag_subshader.compile_subshader();
				// Cache it, and link it
				let frag_subshader = shader_manager.cache_subshader(frag_subshader).unwrap();
				shader.link_subshader(&frag_subshader);
			}
		}

		shader.finalize_shader();
		let cached_shader = shader_manager.cache_shader(shader).unwrap();
		return cached_shader;
	}
	// Finalizes a vert/frag shader by compiling it
	pub fn finalize_shader(&mut self) {
		unsafe {
			// Finalize the shader and stuff
			gl::LinkProgram(self.program);

			// Check for errors
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
					0 as *mut i32,
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
				gl::DetachShader(self.program, subshader_program.clone());
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
		} else {
			println!("Shader '{}' not finalized!", self.name);
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
	pub fn get_uniform_location(&self, name: &str) -> i32 {
		unsafe {
			return gl::GetUniformLocation(self.program, CString::new(name).unwrap().as_ptr());
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
	// Set a texture basically
	pub fn set_texture2d(&self, name: &str, texture_id: u32, active_texture_id: u32) {
		unsafe {
			gl::ActiveTexture(active_texture_id);
			gl::BindTexture(gl::TEXTURE_2D, texture_id);
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
	Geometry,
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
			Resource::Shader(shader) => {
				// Turn the loaded sub shader into a normal sub shader
				let subshader = Self {
					name: shader.name.clone(),
					program: 0,
					source: shader.source.clone(),
					subshader_type: shader.subshader_type.clone(),
				};
				return Some(subshader);
			}
			_ => return None,
		}
	}
	// Compile the current subshader's source code
	pub fn compile_subshader(&mut self) {
		let mut shader_type: u32 = 0;
		match self.subshader_type {
			SubShaderType::Vertex => shader_type = gl::VERTEX_SHADER,
			SubShaderType::Fragment => shader_type = gl::FRAGMENT_SHADER,
			SubShaderType::Geometry => shader_type = gl::GEOMETRY_SHADER,
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
					0 as *mut i32,
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
