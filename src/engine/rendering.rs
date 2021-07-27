use std::collections::HashMap;
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
	// Create a new shader
	pub fn create_shader() -> Shader {
		let mut shader = Shader {
			name: String::from("Unnamed Shader"),
    		linked_subshaders: Vec::new(),
    		program: 0,			
		};
		shader
	}
	// Create a subshader from a loaded subshader resource, then immediatly cache it
	pub fn create_subshader_from_resource(&mut self, resource: &Resource) -> Option<SubShader> {
		match resource {    		
    		Resource::Shader(shader) => {
				// Turn the loaded sub shader into a normal sub shader
				let subshader = SubShader {
					name: shader.name.clone(),
        			program: 0,
        			source: shader.source.clone(),
        			subshader_type: shader.subshader_type.clone(),
    			};
				// Cache the subshader, then load it back from the cache because uh, rust
				self.cache_subshader(&subshader, shader.name.clone());
				return Some(subshader);
			},
    		_ => return None,
		}
	}
	// Caches a specific shader
	fn cache_subshader(&mut self, subshader: &SubShader, subshader_name: String) {
		if !self.subshaders.contains_key(&subshader_name) {
			let mut clone = subshader.clone();
			// Cache the subshader for later use
			self.subshaders.insert(subshader_name, clone);
		} else {
			// Get the cached subshader id
		}
	}
	// Gets a specific subshader from it's name
	pub fn get_subshader(&self, subshader_name: String) -> &SubShader{
		self.subshaders.get(&subshader_name).unwrap()
	}
}

// A shader that contains two sub shaders that are compiled independently
pub struct Shader {
	pub name: String,
	pub linked_subshaders: Vec<String>,
	pub program: u32,
}

impl Shader {
	// Use this shader for rendering a specific entity
	pub fn use_shader(&self) {
		unsafe {
			gl::UseProgram(self.program);
		}
	} 
	// Link a specific subshader to this shader
	pub fn link_subshader(&mut self, subshader_name: String) { 
		self.linked_subshaders.push(subshader_name);
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

	}	
}

// A simple model that holds vertex, normal, and color data
struct Model {
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
	model: Model,
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
			gl::GenBuffers(1, &mut self.gpu_data.vertex_buf);
			gl::BindBuffer(gl::ARRAY_BUFFER, self.gpu_data.vertex_buf);
		}
	}

	// Dispose of our model
	pub fn dispose_model(&mut self) {
		unsafe {
			// Delete the vertex array
			gl::DeleteBuffers(1, &mut self.gpu_data.vertex_buf);
		}
	}

	// Set the model
	pub fn set_model(&mut self, model: Model) {
		self.refresh_model();
	}
	// Get the model
	pub fn get_model(&mut self) -> &mut Model {
		&mut self.model
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