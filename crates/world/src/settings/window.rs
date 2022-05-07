use serde::{Deserialize, Serialize};
// FPS cap
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FrameRateCap {
    Unlimited,
    Limited(usize),
    Vsync,
}

impl Default for FrameRateCap {
    fn default() -> Self {
        Self::Unlimited
    }
}

// Window settings
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct WindowSettings {
    // Main
    pub fullscreen: bool,
    pub fps_cap: FrameRateCap,
}
