use crate::canvas::Canvas;

use super::{Context, get_static_str};

// A device is just some sort of wrapper around glutin windows
pub struct Device {
    // The underlying glutin window
    glutin: glutin::window::Window,

    // Size (in points) of the device window
    size: vek::Extent2<u16>,

    // Fullscreen state of the window
    fullscreen: bool,

    // Raw underlying default canvas
    canvas: Canvas,
}

impl Device {
    // Create a new window using a Glutin window
    pub(crate) fn new(ctx: &mut Context, glutin: glutin::window::Window) -> Self {
        // Glutin window shit
        let size = vek::Extent2::<u32>::from(<(u32, u32)>::from(glutin.inner_size())).as_::<u16>();
        let fullscreen = glutin.fullscreen().is_some();

        // Device creation
        Self {
            glutin,
            size,
            fullscreen,
            canvas: unsafe { Canvas::from_raw_parts(ctx, 0, size) },
        }
    }

    // Get the default window canvas
    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    // Get the raw glutin window
    pub fn window(&self) -> &glutin::window::Window {
        &self.glutin
    }

    // Get the default window canvas mutably
    pub fn canvas_mut(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    // Get the GPU name (renderer)
    pub fn name(&self) -> &'static str {
        unsafe { get_static_str(gl::RENDERER) }
    }

    // Get the vendor (Company responsible for the OpenGL implementation)
    pub fn vendor(&self) -> &'static str {
        unsafe { get_static_str(gl::VENDOR) }
    }
}
