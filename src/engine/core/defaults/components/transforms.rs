// Transforms components
use crate::engine::core::ecs::component::{Component, ComponentID, ComponentInternal};
// The transform component
pub struct Transform {
    pub position: cgmath::Point3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: cgmath::Vector3<f32>,
    pub matrix: cgmath::Matrix4<f32>,
}

// Default transform
impl Default for Transform {
    fn default() -> Self {
        Self {
            position: cgmath::Point3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Point3::new(0.0, 0.0, 0.0),
            scale: glam::Vec3::ONE,
            matrix: glam::Mat4::IDENTITY,
        }
    }
}

// Update the transform matrix
impl Transform {
    // Calculate the matrix and save it
    pub fn update_matrix(&mut self) {
        self.matrix = glam::Mat4::from_translation(self.position) * glam::Mat4::from_quat(self.rotation) * glam::Mat4::from_scale(self.scale);
    }
    // Calculate the matrix and return it
    pub fn get_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_translation(self.position) * glam::Mat4::from_quat(self.rotation) * glam::Mat4::from_scale(self.scale)
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
