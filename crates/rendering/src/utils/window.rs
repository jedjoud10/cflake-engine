use std::sync::atomic::AtomicBool;

// Get the default width and height of the starting window
pub const DEFAULT_WINDOW_SIZE: veclib::Vector2<u16> = veclib::vec2(1280, 720);

// A window class to organize things
pub struct Window {
    pub vsync: bool,
    pub fullscreen: bool,
    pub dimensions: veclib::Vector2<u16>,
    pub focused: bool,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            fullscreen: false,
            vsync: false,
            dimensions: DEFAULT_WINDOW_SIZE,
            focused: false,
        }
    }
}
