// Transforms components
use hypo_ecs::{Component, ComponentID, ComponentInternal};
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
            position: veclib::Vector3::default_zero(),
            rotation: veclib::Quaternion::default_identity(),
            scale: veclib::Vector3::default_one(),
            matrix: veclib::Matrix4x4::default_identity(),
        }
    }
}

// Update the transform matrix
impl Transform {
    // Calculate the matrix and save it
    pub fn update_matrix(&mut self) {
        self.matrix = veclib::Matrix4x4::<f32>::from_translation(self.position)
            * veclib::Matrix4x4::<f32>::from_quaternion(&self.rotation)
            * veclib::Matrix4x4::<f32>::from_scale(self.scale);
    }
    // Calculate the matrix and return it
    pub fn get_matrix(&self) -> veclib::Matrix4x4<f32> {
        return veclib::Matrix4x4::<f32>::from_translation(self.position)
            * veclib::Matrix4x4::<f32>::from_quaternion(&self.rotation)
            * veclib::Matrix4x4::<f32>::from_scale(self.scale);
    }
}

hypo_ecs::impl_component!(Transform);
