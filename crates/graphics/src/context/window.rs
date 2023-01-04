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

// A viewport wrapper around raw Vulkan viewport
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub origin: vek::Vec2<i32>,
    pub extent: vek::Extent2<u32>,
}

// A window is what we will draw to at the end of each frame
pub struct Window {
    settings: WindowSettings,
    raw: winit::window::Window,
    size: vek::Extent2<u32>,
    dirty: bool,
}

impl Window {
    // Create a new window wrapper
    pub(crate) fn new(
        settings: WindowSettings,
        raw: winit::window::Window,
        size: vek::Extent2<u32>,
    ) -> Self {
        Self {
            settings,
            raw,
            size,
            dirty: false,
        }
    }

    // Get the internal window settings
    pub fn settings(&self) -> &WindowSettings {
        &self.settings
    }

    // Get the raw winit window
    pub fn window(&self) -> &winit::window::Window {
        &self.raw
    }

    // Get a Vulkan viewport that represents this window 
    pub fn viewport(&self) -> Viewport {
        Viewport {
            origin: vek::Vec2::default(),
            extent: self.size,
        }
    }

    // Get the current size of the window in pixels
    pub fn size(&self) -> vek::Extent2<u32> {
        self.size
    }

    // Check if the window was resized in the past
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    // Update the window size. Only used internally
    pub(crate) fn set_size(&mut self, size: vek::Extent2<u32>) {
        self.size = size;
        self.dirty = true;
    }

    // Reset the "dirty" state of the window
    pub(crate) fn reset_dirty(&mut self) {
        self.dirty = false;
    }
}
