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
    pub fn update(&mut self) {
        // Update linear then angular physics
        self.linear.update();
    }
}