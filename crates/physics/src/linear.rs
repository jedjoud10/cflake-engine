// Linear momentum
#[derive(Default)]
pub struct LinearPhysics {
    pub acceleration: veclib::Vector3<f32>,
    pub velocity: veclib::Vector3<f32>,
    pub position: veclib::Vector3<f32>,
}

// Update tick
impl LinearPhysics {
    // Update the physics. This must run at a constant tick rate 
    pub fn update(&mut self) {
        // Update acceleration
        // Update velocity
        self.velocity += self.acceleration;
        // Update position
        self.position += self.velocity;
    }
}