use std::ptr::null_mut;

use crate::engine::core::ecs::*;
use nalgebra::Point3;

// Shader manager
pub struct ShaderManager {
	pub shaders: Vec<Shader>,
	pub sub_shaders: Vec<SubShader>,	
}

impl ShaderManager {
	// Create a new shader
	pub fn create_shader() -> Shader {
		let mut shader = Shader {
			name: "Unnamed Shader",
    		vertex_subshader: (),
    		fragment_subshader: (),
    		program: (),			
		}
		shader
	}
	// Loads a subshader
	pub fn load_subshader() -> SubShader {

	}
}

// Default
impl Default for ShaderManager {
	fn default() -> Self {
		Self {
			shaders: Vec::new(),
			sub_shaders: Vec::new(),
		}
	}
}

// A shader that contains two sub shaders that are compiled independently
pub struct Shader {
	pub name: String,
	pub Vec
	pub program: u32,
}

impl Shader {
	pub fn use_shader(&self) {
		unsafe {
			gl::UseProgram(program)
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
pub struct SubShader {
	pub program_id: u32,
	pub source: String,
	pub subshader_type: SubShaderType 
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