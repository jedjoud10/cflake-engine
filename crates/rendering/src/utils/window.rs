// Get the default width and height of the starting window
pub const WINDOW_SIZE: veclib::Vector2<u16> = veclib::vec2(1280, 720);

// A window class to organize things
#[derive(Clone)]
pub struct Window {
    pub vsync: bool,
    pub fullscreen: bool,
    pub dimensions: veclib::Vector2<u16>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            fullscreen: false,
            vsync: false,
            dimensions: WINDOW_SIZE,
        }
    }
}
