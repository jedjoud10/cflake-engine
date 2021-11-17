use others::get_default_window_size;

// A window class to organize things
#[derive(Clone)]
pub struct Window {
    pub fullscreen: bool,
    pub dimensions: veclib::Vector2<u16>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            fullscreen: false,
            dimensions: {
                let d = get_default_window_size();
                veclib::Vector2::new(d.0, d.1)
            },
        }
    }
}
