use ash::vk;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{
    event_loop::EventLoop,
    window::{Fullscreen, WindowBuilder},
};

// Frame rate limit of the window (can be disabled by selecting Unlimited)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FrameRateLimit {
    VSync,
    Limited(u32),
    Umlimited,
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
    settings: WindowSettings,
    raw: winit::window::Window,
    surface: vk::SurfaceKHR,
    surface_loader: ash::extensions::khr::Surface,
}

impl Window {
    // Create a new window using an event loop and it's settings
    pub(crate) unsafe fn new<T>(
        el: &EventLoop<T>,
        window_settings: WindowSettings,
        raw: winit::window::Window,
        instance: &ash::Instance,
        entry: &ash::Entry,
    ) -> Window {
        // Get a window and display handle to the winit window
        let display_handle = raw.raw_display_handle();
        let window_handle = raw.raw_window_handle();

        // Create a surface loader and the surface itself
        let surface = ash_window::create_surface(
            &entry,
            &instance,
            display_handle,
            window_handle,
            None,
        )
        .unwrap();
        let surface_loader =
            ash::extensions::khr::Surface::new(&entry, &instance);

        Self {
            settings: window_settings,
            raw,
            surface,
            surface_loader,
        }
    }

    // Get access to the internal settings this window used during initialization
    pub fn settings(&self) -> &WindowSettings {
        &self.settings
    }

    // Get access to the internal raw winit window
    pub fn raw(&self) -> &winit::window::Window {
        &self.raw
    }

    // Destroy the window after we've done using it
    pub(crate) unsafe fn destroy(self) {
        self.surface_loader.destroy_surface(self.surface, None);
    }
}
