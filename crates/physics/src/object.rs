use crate::{AngularPhysics, LinearPhysics};

// Main physics object class
#[derive(Default, Clone)]
pub struct PhysicsObject {
    pub(crate) mass: f32,
    pub(crate) linear: LinearPhysics,
    pub(crate) angular: AngularPhysics,
    // A locally stored transform
    pub(crate) position: veclib::Vector3<f32>,
    pub(crate) rotation: veclib::Quaternion<f32>,
}

impl PhysicsObject {
    // Update
    pub fn update(&mut self, delta: f32) {
        // Update linear then angular physics
        self.linear.update(&mut self.position, delta);
    }
    // Update the physic object's velocity
    pub fn set_velocity(&mut self, vel: veclib::Vector3<f32>) {
        self.linear.velocity = vel;
    }
    // Get the stored position
    pub fn get_position(&self) -> &veclib::Vector3<f32> {
        &self.position
    }
    // Get the stored rotation
    pub fn get_rotation(&self) -> &veclib::Quaternion<f32> {
        &self.rotation
    }
    // Set the physics object's position.
    pub fn set_position(&mut self, position: veclib::Vector3<f32>) {
        self.position = position;
    }
    // Set the physics object's rotation.
    pub fn set_rotation(&mut self, rotation: veclib::Quaternion<f32>) {
        self.rotation = rotation;
    }
}
