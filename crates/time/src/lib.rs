use std::time::{Duration, Instant};

use world::resources::Resource;

// Global resource that defines the time since the start of the engine and the current frame data
#[derive(Resource)]
#[Locked]
pub struct Time {
    // The difference in seconds between the last frame and the current frame
    delta: f32,

    // How many frames has the engine been running for?
    frame_count: u128,

    // When the engine started
    startup: Instant,

    // The start of the current frame
    frame_start: Instant,
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
    // Update the time with the specified delta
    // This should only be called by the default engine, at the start of each frame
    pub fn update(&mut self, delta: f32) {
        self.delta = delta;
        self.frame_count += 1;
        self.frame_start = Instant::now();
    }

    // Get the delta time
    pub fn delta(&self) -> f32 {
        self.delta
    }

    // Get the frame count
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
