// Static time variables
#[derive(Default)]
pub struct Time {
    pub elapsed: f64,
    pub delta: f64,
    pub frame_count: u64,
}

impl Time {
    /// Update the time
    pub fn update(&mut self, new_time: f64) {
        self.delta = new_time - self.elapsed;
        self.elapsed = new_time;
        self.frame_count += 1;
    }
}
