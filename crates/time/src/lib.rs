use std::time::Instant;

use world::resources::Resource;

// Global resource that defines the time since the start of the engine and the current frame data
pub struct Time {
    // The difference in seconds between the last frame and the current frame
    delta: f64,

    // How many frames has the engine been running for?
    frame_count: u128,

    // When the engine started
    startup: Instant,

    // The start of the current frame
    frame_start: Instant,
}

impl Resource for Time {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn start_frame(&mut self) {
        let now = Instant::now();
        self.delta = (now - self.frame_start).as_secs_f64();
        self.frame_start = now;
        self.frame_count += 1;
    }
}

impl Default for Time {
    fn default() -> Self {
        Self {
            delta: Default::default(),
            frame_count: Default::default(),
            startup: Instant::now(),
            frame_start: Instant::now(),
        }
    }
}

impl Time {
    // Get the time it took to complete one frame
    pub fn delta(&self) -> f64 {
        self.delta
    }

    // Get the total frame count 
    pub fn frame_count(&self) -> u128 {
        self.frame_count
    }

    // Get the moment we started the engine
    pub fn startup(&self) -> Instant {
        self.startup
    }

    // Get the moment the current frame started
    pub fn frame_start(&self) -> Instant {
        self.frame_start
    }
}
