use winit::{
    event_loop::EventLoop,
    window::{Fullscreen, WindowBuilder},
};

// Frame rate limit of the window (can be disabled by selecting Unlimited)
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum FrameRateLimit {
    VSync,
    Limited(u32),

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
    // Create a new window using an event loop and it's settings
    pub(crate) unsafe fn new(
        window_settings: WindowSettings,
        el: &EventLoop<()>,
    ) -> Self {
        let raw = WindowBuilder::default()
            .with_fullscreen(
                window_settings
                    .fullscreen
                    .then_some(Fullscreen::Borderless(None)),
            )
            .with_title(&window_settings.title)
            .build(&el)
            .unwrap();

        Self {
            settings: window_settings,
            raw,
        }
    }
}