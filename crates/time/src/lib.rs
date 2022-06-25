use std::time::Instant;

use world::{Resource, Update, World};

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

    fn removable(world: &mut World) -> bool
    where
        Self: Sized,
    {
        false
    }

    fn inserted(&mut self, world: &mut World) {
        world.events().register_with::<Update>(
            |world: &mut World| {
                let time = world.get_mut::<&mut Self>().unwrap();
                time.frame_count += 1;
                let now = Instant::now();
                time.delta = (now - time.frame_start).as_secs_f64();
                time.frame_start = now;
            },
            i32::MIN,
        )
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
