use std::time::{Duration, Instant};
use world::{Events, Init, Resource, Update, World};

// Global resource that defines the time since the start of the engine and the current frame data
#[derive(Resource)]
pub struct Time {
    // The difference in seconds between the last frame and the current frame
    delta: Duration,

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
    // Get the time it took to complete one frame
    pub fn delta(&self) -> Duration {
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

// The timer system will automatically insert the Time resource and will update it at the start of each frame
pub fn system(events: &mut Events) {
    // Init event (called once at the start of program)
    fn init(world: &mut World) {
        world.insert(Time {
            delta: Duration::ZERO,
            frame_count: 0,
            startup: Instant::now(),
            frame_start: Instant::now(),
        })
    }

    // Update event (called per frame)
    fn update(world: &mut World) {
        let time = world.get_mut::<&mut Time>().unwrap();
        let now = Instant::now();
        time.delta = now - time.frame_start;
        time.frame_start = now;
    }

    // Register the events
    events.registry::<Init>().insert(init);
    events.registry::<Update>().insert(update);
}
