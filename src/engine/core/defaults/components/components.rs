use crate::engine::core::{ecs::*, world::World};
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
		let vertical_fov: f32 = 2.0 * ((self.window_size.1 as f32 / 2.0).tan() * (self.window_size.1 * self.window_size.0) as f32).atan();
		self.projection_matrix = glm::Mat4::new_perspective(self.aspect_ratio, vertical_fov, self.clip_planes.0, self.clip_planes.1);
	}
}

impl Component for Camera {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
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