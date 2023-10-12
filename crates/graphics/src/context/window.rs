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


/// A window wrapper that contains the winit window
pub struct Window {
    pub(crate) raw: winit::window::Window,
    pub(crate) frame_manager: phobos::FrameManager,
    pub(crate) surface: phobos::Surface,
}


impl Window {
    // Get the raw winit window
    pub fn raw(&self) -> &winit::window::Window {
        &self.raw
    }
}
