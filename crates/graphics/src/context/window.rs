use std::sync::Arc;

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

// Format of the swapchain / window presentable texture
//pub type SwapchainFormat = BGRA<Normalized<u8>>;

// A window is what we will draw to at the end of each frame
pub struct Window {
    // Raw winit window and settings
    pub(crate) settings: WindowSettings,
    pub(crate) raw: Arc<winit::window::Window>,
    pub(crate) size: vek::Extent2<u32>,
}

impl Window {
    // Get the internal window settings
    pub fn settings(&self) -> &WindowSettings {
        &self.settings
    }

    // Get the raw winit window
    pub fn raw(&self) -> &winit::window::Window {
        &self.raw
    }

    // Get the current size of the window in pixels
    pub fn size(&self) -> vek::Extent2<u32> {
        self.size
    }
}
