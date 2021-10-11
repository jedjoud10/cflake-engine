// A single value spring, that tries to reach a target with a certain dampening value
#[derive(Default)]
pub struct Spring {
    pub target: f32,
    pub current: f32,
    pub velocity: f32,
    pub dampen: f32,
    pub force: f32,
}

// Actual sring code
impl Spring {
    // Update the spring and return the current spring value
    pub fn update_get(&mut self) -> f32 {
        // Update the velocity
        self.velocity += (self.target - self.current) * self.force;
        // Dampen compensation
        self.velocity += -self.velocity * self.dampen;

        // Update the current value
        self.current = self.velocity + self.current;
        return self.current;
    }
}
