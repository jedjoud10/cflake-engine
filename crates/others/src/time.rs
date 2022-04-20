use std::time::Instant;

use getset::{CopyGetters, Getters};

// Time info about the current frame
#[derive(Clone)]
pub struct FrameTimings {
    // How much time has elapsed in this frame
    pub instant: Instant,
    // The frame count
    pub count: u128,
}
// Static time variables
#[derive(Getters, CopyGetters)]
pub struct Time {
    // Frame
    #[getset(get_copy = "pub")]
    elapsed: f32,
    #[getset(get_copy = "pub")]
    delta: f32,
    #[getset(get = "pub")]
    current: Option<FrameTimings>,

    // Profiler
    #[getset(get_copy = "pub")]
    average_delta: f32,
    averages: [f32; 30],
}

impl Default for Time {
    fn default() -> Self {
        Self {
            elapsed: Default::default(),
            delta: Default::default(),
            average_delta: Default::default(),
            averages: Default::default(),
            current: None,
        }
    }
}

impl Time {
    // Update the time
    pub fn update(&mut self, delta: f32) {
        self.delta = delta;
        self.elapsed += delta;
        // Update current frame
        if self.current.is_none() {
            // First frame ever
            self.current = Some(FrameTimings {
                instant: Instant::now(),
                count: 0,
            });
        } else if let Some(current) = &mut self.current {
            // Update
            current.count += 1;
            current.instant = Instant::now();
        }
        self.averages.rotate_right(1);
        self.averages[0] = delta;

        // Averages
        let sum: f32 = self.averages.iter().cloned().sum();
        self.average_delta = sum / self.averages.len() as f32;
    }
    // Get the current frame count
    pub fn count(&self) -> Option<u128> {
        self.current.as_ref().map(|current| current.count)
    }
}
