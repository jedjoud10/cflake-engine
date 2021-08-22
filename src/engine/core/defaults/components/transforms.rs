// Transforms components
use crate::engine::core::ecs::component::{Component, ComponentID, ComponentInternal};
// The transform component
pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
    pub matrix: glam::Mat4,
}

// Default transform
impl Default for Transform {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
            matrix: glam::Mat4::IDENTITY
        }
    }
}

// Update the transform matrix
impl Transform {
    // Update the matrix
    pub fn update_matrix(&mut self) {
        self.matrix = glam::Mat4::from_translation(self.position) * glam::Mat4::from_quat(self.rotation) * glam::Mat4::from_scale(self.scale);
    }
}

impl ComponentInternal for Transform {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ComponentID for Transform {
    fn get_component_name() -> String {
        String::from("Transform")
    }
}

impl Component for Transform {}
