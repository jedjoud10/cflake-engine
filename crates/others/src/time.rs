// Static time variables
#[derive(Default)]
pub struct Time {
    pub elapsed: f64,
    pub delta: f64,
    pub frame_count: u128,

    pub smoothed_delta: f64,
    last_poll_time: f64
}

impl Time {
    /// Update the time
    pub fn update(&mut self, delta: f64) {
        self.delta = delta;
        self.elapsed += delta;
        self.frame_count += 1;

        // Polling
        if self.elapsed > self.last_poll_time {
            self.last_poll_time += 0.25;
            self.smoothed_delta = delta;
        }
    }
}
