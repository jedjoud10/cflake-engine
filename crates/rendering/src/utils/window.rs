use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use glutin::window::Fullscreen;

// Get the default width and height of the starting window
pub const DEFAULT_WINDOW_SIZE: veclib::Vector2<u16> = veclib::vec2(1280, 720);

// A window class to organize things
pub struct Window {
    pub dimensions: veclib::Vector2<u16>,
    pub window: Option<glutin::window::Window>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            dimensions: DEFAULT_WINDOW_SIZE,
            window: Default::default(),
        }
    }
}

impl Window {
    // These methods MUST be called on the main thread
    // Enable/disable fullscreen for the window
    pub fn set_fullscreen(&self, fullscreen: bool) {
        // Panic if we try to do on any other thread other than the main thread
        if !others::on_main_thread() {
            panic!("We cannot update the window settings if we are not on the main thead!");
        }
        let window = self.window.as_ref().unwrap();
        if fullscreen {
            let vm = window.primary_monitor().unwrap().video_modes().nth(0).unwrap();
            window.set_fullscreen(Some(Fullscreen::Exclusive(vm)));
        } else {
            window.set_fullscreen(None);
        }
    }
}
