use crate::engine::core::ecs::Component;

pub struct Position {
	x: f32, y: f32, z: f32
}

impl Component for Position {
	fn get_component_name() -> String {
		String::from("Position")
	}
}