// Static time variables
#[derive(Default)]
pub struct Time {
    pub seconds_since_game_start: f64,
    pub delta_time: f64,
    pub frame_count: u64,
    pub fps: f64,
    pub average_fps: f64,
    // Used for the one second tick
    last_tick_time: f64,
}

impl Time {
    // Update the average FPS
    pub fn update_average_fps(&mut self) {
        if self.last_tick_time < self.seconds_since_game_start {
            // Update again and set the average fps
            self.average_fps = self.fps;
            self.last_tick_time = self.seconds_since_game_start + 0.5;
        }
    }
}
