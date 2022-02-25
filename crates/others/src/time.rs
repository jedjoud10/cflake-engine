use std::time::Instant;

// Time info about the current frame
pub struct FrameTimings {
    // How much time has elapsed in this frame
    pub begin_instant: Instant,
    // The frame count
    pub count: u128,
}

// Static time variables
pub struct Time {
    // How much time has elapsed since the start of the game
    pub elapsed: f64,
    // Delta
    pub delta: f64,
    // Frame
    pub current: FrameTimings,
}

impl Default for Time {
    fn default() -> Self {
        Self { 
            elapsed: Default::default(),
            delta: Default::default(),
            current: FrameTimings {
                begin_instant: Instant::now(),
                count: 0,
            }
        }
    }
}

impl Time {
    // Update the time
    pub fn update(&mut self, delta: f64) {
        self.delta = delta;
        self.elapsed += delta;
        // Update current frame
        self.current.count += 1;
        self.current.begin_instant = Instant::now();
    }
    // Late update
    pub fn late_update(&mut self) {
    }
    // Update the current frame time
    pub fn update_current_frame_time(&mut self) {  
        self.current.begin_instant = Instant::now();      
    }
}
