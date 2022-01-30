// Linear momentum
#[derive(Default, Clone)]
pub struct LinearPhysics {
    pub acceleration: veclib::Vector3<f32>,
    pub velocity: veclib::Vector3<f32>,
}

// Update tick
impl LinearPhysics {
    // Update the physics. This must run at a constant tick rate
    pub fn update(&mut self, position: &mut veclib::Vector3<f32>, delta: f32) {
        // Update acceleration
        // Update velocity
        //self.acceleration += (veclib::vec3(0.0, GRAVITY_ACCELERATION, 0.0) * delta) * 0.1;
        self.velocity += self.acceleration * delta;
        // Update position
        *position += self.velocity * delta;
    }
}
