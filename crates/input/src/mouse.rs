use world::Resource;

// This mouse struct will be responsible for all the values that require the mouse, like cursor position or position delta
#[derive(Resource)]
pub struct Mouse {
    // Absolute values (not clampled)
    pub(crate) scroll_delta: f32,
    pub(crate) scroll: f32,

    // Delta differences since last frame
    pub(crate) pos_delta: vek::Vec2<f32>,
    pub(crate) pos: vek::Vec2<f32>,
}

impl Mouse {
    // Get the scroll wheel delta (difference since last frame)
    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }

    // Get the scroll wheel absolute value
    pub fn scroll(&self) -> f32 {
        self.scroll
    }

    // Get the mouse position delta (difference since last frame)
    pub fn position_delta(&self) -> vek::Vec2<f32> {
        self.pos_delta
    }

    // Get the absolute mouse position (not screen campled)
    pub fn position(&self) -> vek::Vec2<f32> {
        self.pos
    }
}
