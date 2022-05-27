use crate::framebuffer::{Canvas, Framebuffer};

use super::Context;

// A device is just some sort of wrapper around glutin windows
pub struct Device {
    // The underlying glutin window
    glutin: glutin::window::Window,

    // Size (in points) of the device window
    size: vek::Extent2<u16>,

    // Fullscreen state of the window
    fullscreen: bool,

    // Raw underlying default framebuffer
    framebuffer: Framebuffer,
} 

impl Device {
    // Create a new window using a Glutin window
    pub(crate) fn new(glutin: glutin::window::Window) -> Self {
        // Convert the size into a tuple
        let size = vek::Extent2::<u32>::from(<(u32, u32)>::from(glutin.inner_size())).as_::<u16>();
        let fullscreen = glutin.fullscreen().is_some();

        Self { glutin, size, fullscreen, framebuffer: unsafe { Framebuffer::from_raw_parts(0, size) } }
    }

    // Get the default canvas for the window
    pub fn canvas(&mut self, ctx: &mut Context) -> Canvas {
        self.framebuffer.canvas(ctx)
    }
}