use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

// Window init settings
pub struct WindowInitSettings {
    pub dimensions: veclib::Vector2<u16>,
    pub pixel_per_point: f32,
}

// Get the default width and height of the starting window
pub const DEFAULT_WINDOW_SIZE: veclib::Vector2<u16> = veclib::vec2(1280, 720);

// A window class to organize things
#[derive(Default)]
pub struct Window {
    pub dimensions: veclib::Vector2<u16>,
    pub focused: bool,
    pub(crate) vsync: AtomicBool,
    pub(crate) update: AtomicBool,
}

impl Window {
    // Create a new window
    pub fn new() -> Self {
        Self {
            dimensions: DEFAULT_WINDOW_SIZE,
            vsync: AtomicBool::new(false),
            update: AtomicBool::new(false),
            focused: false,
        }
    }
    // These methods MUST be called on the main thread
    // Enable/disable fullscreen for the window
    pub fn set_fullscreen(&self, fullscreen: bool) {
        // Panic if we try to do on any other thread other than the main thread
        if !others::on_main_thread() {
            panic!("We cannot update the window settings if we are not on the main thead!");
        }
        if fullscreen {
        } else {
        }
    }
    // Enable or disable vsync
    pub fn set_vsync(&self, vsync: bool) {
        // Panic if we try to do on any other thread other than the main thread
        if !others::on_main_thread() {
            panic!("We cannot update the window settings if we are not on the main thead!");
        }
        self.update.store(true, Ordering::Relaxed);
        self.vsync.store(vsync, Ordering::Relaxed);
    }
}
