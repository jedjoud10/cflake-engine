use crate::engine::core::ecs::Component;
use crate::engine::core::ecs::ComponentNames;

pub struct Position {
	x: f32, y: f32, z: f32
}

impl Component for Position {
}

impl ComponentNames for Position {
	fn get_component_name() -> String {
		String::from("Position")
	}
}

impl Default for Position {
	fn default() -> Self {
		Self {
			x: 0.0,
			y: 0.0,
			z: 0.0,
		}
	}
}