// Static time variables
#[derive(Default)]
pub struct Time {
    pub seconds_since_game_start: f64,
    pub delta_time: f64,
    pub frame_count: u64,
    pub fps: f64,
    pub average_fps: f64,
    pub average_fps_velocity: f64,
}

impl Time {
    // Update the average FPS
    pub fn update_average_fps(&mut self) {
        self.average_fps_velocity = self.fps - self.average_fps;
        self.average_fps += self.average_fps_velocity * 0.2 - self.average_fps_velocity * 0.01;
    }
}
