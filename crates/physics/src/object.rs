use crate::{AngularPhysics, LinearPhysics};

// Main physics object class
#[derive(Default, Clone)]
pub struct PhysicsObject {
    pub mass: f32,
    pub linear: LinearPhysics,
    pub angular: AngularPhysics,
}

impl PhysicsObject {
    // Update
    pub fn update(&mut self, position: &mut veclib::Vector3<f32>, _rotation: &mut veclib::Quaternion<f32>, delta: f32) {
        // Update linear then angular physics
        self.linear.update(position, delta);
    }
    // Update the physic object's velocity
    pub fn set_velocity(&mut self, vel: veclib::Vector3<f32>) {
        self.linear.velocity = vel;
    }
}
