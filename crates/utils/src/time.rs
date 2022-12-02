use std::time::{Duration, Instant};
use world::{user, System, World};

// Global resource that defines the time since the start of the engine and the current frame data
pub struct Time {
    // The difference in seconds between the last frame and the current frame
    pub(crate) delta: Duration,

    // How many frames has the engine been running for?
    pub(crate) frame_count: u128,

    // When the engine started
    pub(crate) startup: Instant,

    // The start of the current frame
    pub(crate) frame_start: Instant,
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
    pub fn delta(&self) -> Duration {
        self.delta
    }

    // Get the time delta as a float depicting the number of seconds that elapsed
    pub fn delta_f32(&self) -> f32 {
        self.delta().as_secs_f32()
    }

    // Get the total frame count
    pub fn frame_count(&self) -> u128 {
        self.frame_count
    }

    // Get the moment we started the engine
    pub fn startup(&self) -> Instant {
        self.startup
    }

    // Caclulate the number of seconds that have passed since the start of the engine
    pub fn secs_since_startup_f32(&self) -> f32 {
        (Instant::now() - self.startup()).as_secs_f32()
    }

    // Get the moment the current frame started
    pub fn frame_start(&self) -> Instant {
        self.frame_start
    }
}
