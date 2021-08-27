// Transforms components
use crate::engine::core::ecs::component::{Component, ComponentID, ComponentInternal};
use veclib::*;
// The transform component
pub struct Transform {
    pub position: veclib::Vector3<f32>,
    pub rotation: veclib::Quaternion<f32>,
    pub scale: veclib::Vector3<f32>,
    pub matrix: veclib::Matrix4x4<f32>,
}

// Default transform
impl Default for Transform {
    fn default() -> Self {
        Self {
            position: veclib::Vector3::ZERO,
            rotation: veclib::Quaternion::IDENTITY,
            scale: veclib::Vector3::ONE,
            matrix: veclib::Matrix4x4::IDENTITY,
        }
    }
}

// Update the transform matrix
impl Transform {
    // Calculate the matrix and save it
    pub fn update_matrix(&mut self) {
        //self.matrix = veclib::Matrix4x4::from_translation(self.position) * veclib::Matrix4x4::from_quaternion(&self.rotation) * veclib::Matrix4x4::from_scale(self.scale);
    }
    // Calculate the matrix and return it
    pub fn get_matrix(&self) -> veclib::Matrix4x4<f32> {
        todo!();
        //glam::Mat4::from_translation(self.position) * glam::Mat4::from_quat(self.rotation) * glam::Mat4::from_scale(self.scale)
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
