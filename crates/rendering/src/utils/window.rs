use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

use super::RenderWrapper;

// Get the default width and height of the starting window
pub const DEFAULT_WINDOW_SIZE: veclib::Vector2<u16> = veclib::vec2(1280, 720);

// A window class to organize things
#[derive(Default)]
pub struct Window {
    pub dimensions: veclib::Vector2<u16>,
    pub focused: bool,
    pub wrapper: Arc<RenderWrapper>,
}

impl Window {
    // Create a new window
    pub fn new(wrapper: Arc<RenderWrapper>) -> Self {
        Self {
            dimensions: DEFAULT_WINDOW_SIZE,
            focused: false,
            wrapper
        }
    }
    // These methods MUST be called on the main thread
    // Enable/disable fullscreen for the window
    pub fn set_fullscreen(&self, fullscreen: bool) {
        // Panic if we try to do on any other thread other than the main thread
        if !others::on_main_thread() {
            panic!("We cannot update the window settings if we are not on the main thead!");
        }
        let (glfw, window) = (self.wrapper.0.load(Ordering::Relaxed), self.wrapper.1.load(Ordering::Relaxed));
        let (glfw, window) = unsafe { (&mut *glfw, &mut *window) };
        if fullscreen {
            // Set the glfw window as a fullscreen window
            glfw.with_primary_monitor_mut(|_glfw2, monitor| {
                let videomode = monitor.unwrap().get_video_mode().unwrap();
                window.set_monitor(glfw::WindowMode::FullScreen(monitor.unwrap()), 0, 0, videomode.width, videomode.height, Some(videomode.refresh_rate));
                unsafe { gl::Viewport(0, 0, videomode.width as i32, videomode.height as i32); }
            });
        } else {
            // Set the glfw window as a windowed window
            glfw.with_primary_monitor_mut(|_glfw2, monitor| {
                let videomode = monitor.unwrap().get_video_mode().unwrap();
                let default_window_size = crate::utils::DEFAULT_WINDOW_SIZE;
                window.set_monitor(glfw::WindowMode::Windowed, 50, 50, default_window_size.x as u32, default_window_size.y as u32, Some(videomode.refresh_rate));
                unsafe { gl::Viewport(0, 0, default_window_size.x as i32, default_window_size.y as i32); }
            });
        }
    }
    // Enable or disable vsync
    pub fn set_vsync(&self, vsync: bool) {
        // Panic if we try to do on any other thread other than the main thread
        if !others::on_main_thread() {
            panic!("We cannot update the window settings if we are not on the main thead!");
        }
        let (glfw, window) = (self.wrapper.0.load(Ordering::Relaxed), self.wrapper.1.load(Ordering::Relaxed));
        let (glfw, window) = unsafe { (&mut *glfw, &mut *window) };
        if vsync {
            glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
        } else {
            glfw.set_swap_interval(glfw::SwapInterval::None);
        }
    }
}