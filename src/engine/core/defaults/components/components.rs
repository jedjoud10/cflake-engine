use crate::engine::core::{ecs::*, world::World};
use crate::engine::core::defaults::components::transforms::{Position, Rotation};
use crate::engine::rendering::{EntityRenderState, Model, ModelDataGPU};
extern crate nalgebra_glm as glm;


// A simple camera component
pub struct Camera {
	pub view_matrix: glm::Mat4,
	pub projection_matrix: glm::Mat4,
	pub horizontal_fov: f32, 
	pub aspect_ratio: f32,
	pub clip_planes: (f32, f32), // Near, far
	pub window_size: (i32, i32) // Width, height
}

// Impl block for Camera component
impl Camera {
	// Update the projection matrix of this camera
	pub fn update_projection_matrix(&mut self) {
		// Turn the horizontal fov into a vertical one
		let vertical_fov: f32 = 2.0 * ((self.horizontal_fov.to_radians() / 2.0).tan() * (self.window_size.1 * self.window_size.0) as f32).atan();
		self.projection_matrix = glm::Mat4::new_perspective(self.aspect_ratio, vertical_fov, self.clip_planes.0, self.clip_planes.1);
	}
	// Update the view matrix using a rotation and a position
	pub fn update_view_matrix(&mut self, position: &glm::Vec3, rotation: &glm::Quat) {
		let forward_vector: glm::Vec3 =  glm::quat_rotate_vec3(&rotation, &glm::vec3(0.0, 0.0, 1.0));
		let up_vector: glm::Vec3 = glm::quat_rotate_vec3(&rotation, &glm::vec3(0.0, 1.0, 0.0));
		let new_view_matrix = glm::look_at(&position, &forward_vector, &up_vector);
		self.view_matrix = new_view_matrix; 
	}
}

impl Component for Camera {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ComponentID for Camera {
    fn get_component_name() -> String {
        String::from("Camera Data")
    }
}

impl Default for Camera {
	fn default() -> Self {
		Self {
			view_matrix: glm::Mat4::identity(),
			projection_matrix: glm::Mat4::identity(),
			horizontal_fov: 90.0,
    		aspect_ratio: 16.0 / 9.0,
    		clip_planes: (0.0, 1000.0),
    		window_size: World::get_default_window_size(),
		}
	}
}

// A component that will be linked to entities that are renderable
#[derive(Default)]
pub struct Render {
	pub render_state: EntityRenderState,
	pub gpu_data: ModelDataGPU,	
	pub shader_name: String,
	pub model: Model,
}

// Main traits implemented
impl Component for Render {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
	
}
impl ComponentID for Render {
	fn get_component_name() -> String {
		String::from("Render Component")
	}
}