use std::sync::Arc;

use wgpu::{Surface, SurfaceConfiguration, SurfaceCapabilities};

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

// A viewport wrapper around raw WGPU viewport
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub origin: vek::Vec2<i32>,
    pub extent: vek::Extent2<u32>,
}

// A window is what we will draw to at the end of each frame
pub struct Window {
    // Raw winit window and settings
    pub(crate) settings: WindowSettings,
    pub(crate) raw: Arc<winit::window::Window>,
    pub(crate) size: vek::Extent2<u32>,

    // WGPU surface and config
    pub(crate) surface: Surface,
    pub(crate) surface_config: SurfaceConfiguration,
    pub(crate) surface_capabilities: SurfaceCapabilities,
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
}
