use crate::engine::core::ecs::*;
use nalgebra::Point3;

// Shader manager
pub struct ShaderManager {
	pub shaders: Vec<Shader>,
	pub sub_shaders: Vec<SubShader>,	
}

impl ShaderManager {

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
	pub vertex_subshader: u16,
	pub fragment_subshader: u16,
}

// Sub shader type
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

// A component that will be linked to entities that are renderable
pub struct RenderComponent {
	pub render_state: EntityRenderState,
	pub vertex_buffer: *mut u32,
	model: Model,
}

impl RenderComponent {
	// When we update the model and want to refresh it's OpenGL data
	pub fn refresh_model(&mut self) {
		unsafe {
			// Create the vertex buffer and populate it
			gl::GenBuffers(1, self.vertex_buffer);
			gl::BindBuffer(gl::ARRAY_BUFFER, *self.vertex_buffer);
		}
	}

	// Dispose of our model
	pub fn dispose_model(&mut self) {
		unsafe {
			// Delete the vertex array
			gl::DeleteBuffers(1, self.vertex_buffer);
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

impl Default for RenderComponent {
	fn default() -> Self {
		let mut num = 0;
		Self {
			render_state: EntityRenderState::Visible,
			vertex_buffer: &mut num,
			model: Model::default(),
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