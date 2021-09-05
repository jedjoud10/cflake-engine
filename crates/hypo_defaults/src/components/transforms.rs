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

// Transform creation
impl Transform {
    // With position
    pub fn with_position(mut self, position: veclib::Vector3<f32>) -> Self {
        self.position = position;
        self
    }
    // With rotation
    pub fn with_rotation(mut self, rotation: veclib::Quaternion<f32>) -> Self {
        self.rotation = rotation;
        self
    }
    // With scale
    pub fn with_scale(mut self, scale: veclib::Vector3<f32>) -> Self {
        self.scale = scale;
        self
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
