use std::time::Instant;

// Time info about the current frame
#[derive(Clone)]
pub struct FrameTimings {
    // How much time has elapsed in this frame
    pub instant: Instant,
    // The frame count
    pub count: u128,
}
// Static time variables
pub struct Time {
    // Frame
    pub elapsed: f32,
    pub delta: f32,
    pub current: Option<FrameTimings>,

    // Profiler
    pub average_delta: f32,
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
        if let None = self.current {
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
}
