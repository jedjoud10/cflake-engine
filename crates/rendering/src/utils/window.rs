use glutin::window::Fullscreen;

// Get the default width and height of the starting window
pub const DEFAULT_WINDOW_SIZE: veclib::Vector2<u16> = veclib::vec2(1280, 720);

// A window class to organize things
pub struct Window {
    pub dimensions: veclib::Vector2<u16>,
    pub inner: Option<glutin::window::Window>,
    pub pixels_per_point: f64,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            dimensions: DEFAULT_WINDOW_SIZE,
            inner: Default::default(),
            pixels_per_point: Default::default(),
        }
    }
}

impl Window {
    // These methods MUST be called on the main thread
    // Enable/disable fullscreen for the window
    pub fn set_fullscreen(&self, fullscreen: bool) {
        // Panic if we try to do on any other thread other than the main thread
        if !others::on_main_thread() {
            log::error!("We cannot update the window settings if we are not on the main thead!");
            panic!();
        }
        let window = self.inner.as_ref().unwrap();
        if fullscreen {
            let vm = window
                .primary_monitor()
                .unwrap()
                .video_modes()
                .next()
                .unwrap();
            window.set_fullscreen(Some(Fullscreen::Exclusive(vm)));
        } else {
            window.set_fullscreen(None);
        }
    }
}
