use std::sync::Arc;
use winit::{
    event_loop::EventLoop,
    window::{Fullscreen, WindowBuilder},
};

// Frame rate limit of the window (can be disabled by selecting Unlimited)
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum FrameRateLimit {
    // Limit the FPS to the screen refresh rate and use VSync
    VSync,

    // Limit the FPS to a specific value
    Limited(u32),

    // There is no FPS cap
    #[default]
    Unlimited,
}

// Window setting that will tell Winit how to create the window
#[derive(Clone)]
pub struct WindowSettings {
    pub title: String,
    pub fullscreen: bool,
    pub limit: FrameRateLimit,
}

// A window is what we will draw to at the end of each frame
pub struct Window {
    pub(crate) settings: WindowSettings,
    pub(crate) raw: winit::window::Window,
}

impl Window {
    // Get the internal window settings
    pub fn settings(&self) -> &WindowSettings {
        &self.settings
    }

    // Get the raw winit window
    pub fn window(&self) -> &winit::window::Window {
        &self.raw
    }
}
