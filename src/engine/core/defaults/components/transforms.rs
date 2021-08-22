// Transforms components
use crate::engine::core::ecs::component::{Component, ComponentID, ComponentInternal};

// A position component telling us where the entity is in the world
#[derive(Default)]
pub struct Position {
    pub position: glam::Vec3,
}

impl ComponentInternal for Position {
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

impl Component for Position {}

// Scale component
pub struct Scale {
    pub scale: f32,
}

impl ComponentInternal for Scale {
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
        Self { scale: 1.0 }
    }
}

impl Component for Scale {}

// Rotation component
#[derive(Default)]
pub struct Rotation {
    pub rotation: glam::Quat,
}

impl ComponentInternal for Rotation {
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

impl Component for Rotation {}
