use crate::engine::core::ecs::Component;
use crate::engine::core::ecs::ComponentID;

pub struct Position {
	x: f32, y: f32, z: f32
}

impl Component for Position {
}

impl ComponentID for Position {
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

pub struct Scale {
	scale: f32
}

impl Component for Scale {
}

impl ComponentID for Scale {
	fn get_component_name() -> String {
		String::from("Scale")
	}
}

impl Default for Scale {
	fn default() -> Self {
		Self {
			scale: 1.0,
		}
	}
}

pub struct Rotation {
	x: f32,
	y: f32,
	z: f32,
	w: f32,
}

impl Component for Rotation {
}

impl ComponentID for Rotation {
	fn get_component_name() -> String {
		String::from("Rotation")
	}
}

impl Default for Rotation {
	fn default() -> Self {
		Self {
			x: 0.0, y: 0.0, z: 0.0, w: 0.0,
		}
	}
}