use getset::{Getters, MutGetters, CopyGetters};
use glutin::window::Fullscreen;

// Get the default width and height of the starting window
pub const DEFAULT_WINDOW_SIZE: veclib::Vector2<u16> = veclib::vec2(1280, 720);

// A window class to organize things
#[derive(Getters, CopyGetters, MutGetters)]
pub struct Window {
    #[getset(get_copy = "pub")]
    dimensions: veclib::Vector2<u16>,
    #[getset(get = "pub", get_mut = "pub")]
    inner: glutin::window::Window,
    #[getset(get_copy = "pub")]
    pixels_per_point: f64,
    #[getset(get_copy = "pub")]
    fullscreen: bool,
}

impl Window {
    // Enable/disable fullscreen for the window
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        if fullscreen {
            // Enable fullscreen
            let vm = self.inner.primary_monitor().unwrap().video_modes().next().unwrap();
            self.inner.set_fullscreen(Some(Fullscreen::Exclusive(vm)));
        } else {
            // Disable fullscreen
            self.inner.set_fullscreen(None);
        }
        self.fullscreen = fullscreen;
    }
}
