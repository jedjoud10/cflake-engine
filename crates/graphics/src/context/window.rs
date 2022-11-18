use winit::{event_loop::EventLoop, window::{WindowBuilder, Fullscreen}};

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
}

impl Window {
    // Create a new window using an event loop and it's settings
    pub(crate) fn new<T>(el: &EventLoop<T>, settings: WindowSettings) -> Window {
        let window = WindowBuilder::default()
            .with_fullscreen(settings.fullscreen
                .then_some(Fullscreen::Borderless(None)))
            .with_title(&settings.title)
            .build(&el).unwrap();

        Self {
            settings,
            raw: window,
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
}
