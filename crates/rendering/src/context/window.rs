use winit::event_loop::EventLoop;

// Frame rate limit of the window (can be disabled by selecting Unlimited)
pub enum FrameRateLimit {
    VSync,
    Limited(u32),
    Umlimited,
}

// Window setting that will tell Winit how to create the window
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

// Create a new window using an event loop and it's settings
pub(crate) fn new<T>(el: &EventLoop<T>, settings: WindowSettings) -> Window {
    todo!()
}
