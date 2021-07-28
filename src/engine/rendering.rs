use std::{collections::HashMap, ffi::{CString, c_void}, ptr::null};
use crate::engine::core::ecs::*;
use nalgebra::Point3;
use crate::engine::resources::Resource;

// Shader manager
pub struct ShaderManager {
	pub shaders: HashMap<String, Shader>,
	pub subshaders: HashMap<String, SubShader>,	
}

// Default
impl Default for ShaderManager {
	fn default() -> Self {
		Self {
			shaders: HashMap::new(),
			subshaders: HashMap::new(),
		}
	}
}

impl ShaderManager {	
	// Create a subshader from a loaded subshader resource, then immediatly cache it
	pub fn create_subshader_from_resource(&mut self, resource: &Resource) -> Option<SubShader> {
		match resource {    		
    		Resource::Shader(shader) => {
				// Turn the loaded sub shader into a normal sub shader
				let mut subshader = SubShader {
					name: shader.name.clone(),
        			program: 0,
        			source: shader.source.clone(),
        			subshader_type: shader.subshader_type.clone(),
    			};
				return Some(subshader);
			},
    		_ => return None,
		}
	}
	// Caches a specific shader
	pub fn cache_subshader(&mut self, subshader: &SubShader, subshader_name: String) {
		if !self.subshaders.contains_key(&subshader_name) {
			let mut clone = subshader.clone();
			// Cache the subshader for later use
			self.subshaders.insert(subshader_name, clone);
		} else {
			// Well the subshader is already cached so don't do anything
		}
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
				linked_subshaders_programs: Vec::new()
			}
		}
	}
}

impl Shader {
	// Use this shader for rendering a specific entity
	pub fn use_shader(&mut self) {
		// Check if the program even was finalized and ready for use
		if self.finalized {
			unsafe {
				gl::UseProgram(self.program);
			}
		} else {
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
					gl::GetProgramInfoLog(self.program, info_log_length, 0 as *mut i32, log.as_mut_ptr());
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
	} 
	// Link a specific subshader to this shader
	pub fn link_subshader(&mut self, subshader: &SubShader) { 
		self.linked_subshaders_programs.push(subshader.program);
		unsafe {
			gl::AttachShader(self.program, subshader.program);
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
				gl::GetShaderInfoLog(self.program, info_log_length, 0 as *mut i32, log.as_mut_ptr());
				println!("Error while compiling sub-shader {}!:", self.name);
				let printable_log: Vec<u8> = log.iter().map(|&c| c as u8).collect(); 
				let string = String::from_utf8(printable_log).unwrap();
				println!("Error: \n\x1b[31m{}", string);
				println!("\x1b[0m");
				panic!();
			}

			println!("\x1b[32mSubshader {} compiled succsessfully!\x1b[0m", self.name);
		}
	}	
}

// A simple model that holds vertex, normal, and color data
pub struct Model {
	pub vertices: Vec<Point3<f32>>,
	pub triangles: Vec<u32>,
}

impl Default for Model {
	fn default() -> Self {
		Self {
			vertices: Vec::new(),
			triangles: Vec::new(),
		}
	}
}

// The current render state of the entity
pub enum EntityRenderState {
	Visible,
	Invisible,
}

impl Default for EntityRenderState {
	fn default() -> Self { Self::Visible }
}

// A component that will be linked to entities that are renderable
#[derive(Default)]
pub struct RenderComponent {
	pub render_state: EntityRenderState,
	pub gpu_data: ModelDataGPU,	
	pub shader_name: String,
	pub model: Model,
}

// Struct that hold the model's information from OpenGL
#[derive(Default)]
pub struct ModelDataGPU {
	pub vertex_buf: u32,
}

impl RenderComponent {
	// When we update the model and want to refresh it's OpenGL data
	pub fn refresh_model(&mut self) {
		unsafe {
			// Create the vertex buffer and populate it
			let temp_array = vec![
				-1.0, -1.0, 0.0,
				1.0, -1.0, 0.0,
				0.0,  1.0, 0.0];
			gl::GenBuffers(1, &mut self.gpu_data.vertex_buf);
			gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.vertex_buf);
			gl::BufferData(self.gpu_data.vertex_buf, 1, self.model.vertices.as_ptr() as *const c_void, gl::STATIC_DRAW);

			// Create the vertex attrib arrays
			gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.vertex_buf);
			gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());			
			gl::EnableVertexAttribArray(0);
		}
	}

	// Dispose of our model
	pub fn dispose_model(&mut self) {
		unsafe {
			// Delete the vertex array
			gl::DeleteBuffers(1, &mut self.gpu_data.vertex_buf);
		}
	}
}

// Main traits implemented
impl Component for RenderComponent {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl ComponentID for RenderComponent {
	fn get_component_name() -> String {
		String::from("Renderable Component")
	}
}