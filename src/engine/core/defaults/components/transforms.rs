use core::f64;

// Transforms components
use crate::engine::core::ecs::Component;
use crate::engine::core::ecs::ComponentID;
use nalgebra_glm as glm;


// A position component telling us where the entity is in the world
pub struct Position {
	pub position: glm::Vec3
}

impl Component for Position {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ComponentID for Position {
	fn get_component_name() -> String {
		String::from("Position")
	}
}

impl Default for Position {
	fn default() -> Self {
		Self {
			position: glm::vec3(0.0, 0.0, 0.0),
		}
	}
}

// Scale component
pub struct Scale {
	pub scale: f32
}

impl Component for Scale {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
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

// Rotation component
pub struct Rotation {
	pub rotation: glm::Quat
}

impl Component for Rotation {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ComponentID for Rotation {
	fn get_component_name() -> String {
		String::from("Rotation")
	}
}

impl Default for Rotation {
	fn default() -> Self {
		Self {
			rotation: glm::Quat::identity(),
		}
	}
}