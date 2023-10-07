use std::sync::Arc;

/// Frame rate limit of the window (can be disabled by selecting Unlimited)
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum FrameRateLimit {
    /// Limit the FPS to the screen refresh rate and use VSync
    VSync,

    /// Limit the FPS to a specific value
    Limited(u32),

    /// There is no FPS cap
    #[default]
    Unlimited,
}

/// Window setting that will tell Winit how to create the window
#[derive(Clone)]
pub struct WindowSettings {
    /// The title of the window that we will create
    /// You can change this later on with the created winit window
    pub title: String,

    /// The initial fullscreen state of the window
    pub fullscreen: bool,

    /// The initial frame rate limit of the window 
    pub limit: FrameRateLimit,
}