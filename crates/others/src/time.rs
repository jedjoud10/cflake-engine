// Static time variables
#[derive(Default)]
pub struct Time {
    pub elapsed: f64,
    pub delta: f64,
    pub frame_count: u64,
}

impl Time {
    /// Update the time
    pub fn update(&mut self, delta: f64) {
        self.delta = delta;
        self.elapsed += delta;
        self.frame_count += 1;
    }
}
