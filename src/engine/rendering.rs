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
	// Get the ID of a specific subshader
	pub fn generate_subshader_id_cache(&mut self, subshader: SubShader) -> u16 {
		if !self.subshaders.contains_key(&subshader.name) {
			let mut clone = subshader.clone();
			// Cache the subshader for later use
			let id = self.subshaders.len() as u16;
			clone.id = id;
			self.subshaders.insert(subshader.name, clone);
			return id;	
		} else {
			// Get the cached subshader id
			return self.subshaders.get(&subshader.name).unwrap().id;
		}
	}
	
}

// A shader that contains two sub shaders that are compiled independently
pub struct Shader {
	pub name: String,
	pub linked_subshaders: Vec<u16>,
	pub program: u32,
}

impl Shader {
	// Use this shader for rendering a specific entity
	pub fn use_shader(&self) {
		unsafe {
			gl::UseProgram(self.program);
		}
	} 
	// Link a specific subshader to this shader, but first cache it
	pub fn link_subshader(&mut self, shader_manager: &mut ShaderManager, subshader: SubShader) { 
		self.linked_subshaders.push(shader_manager.generate_subshader_id_cache(subshader));
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
	pub id: u16,
}

impl SubShader {
	// Compile the current subshader's source code
	pub fn compile_subshader(&mut self) {

	}
	// Create a subshader from a loaded subshader resource
	pub fn new_from_resource(resource: Resource) -> Option<SubShader> {
		match resource {    		
    		Resource::Shader(shader) => {
				// Turn the loaded sub shader into a normal sub shader
				let subshader = SubShader {
					name: shader.name,
        			program: 0,
					id: 0,
        			source: shader.source,
        			subshader_type: shader.subshader_type,
    			};
				return Some(subshader);
			},
    		_ => return None,
		}
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
	pub shader_id: u16,
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