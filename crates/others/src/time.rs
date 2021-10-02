// Static time variables
#[derive(Default)]
pub struct Time {
    pub seconds_since_game_start: f64,
    pub delta_time: f64,
    pub frame_count: u64,
    pub fps: f64,
    pub average_fps: f64,
}
