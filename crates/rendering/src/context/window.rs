use super::{get_static_str, Context};
use crate::{display::Display, prelude::Viewport};

// A device is a software/hardware renderer that will be responsible for dealing with a specific window
pub struct Device {
    name: &'static str,
    vendor: &'static str,
}

impl Device {
    // Get the GPU name (renderer)
    pub fn name(&self) -> &'static str {
        self.name
    }

    // Get the vendor name (Company responsible for the OpenGL implementation)
    pub fn vendor(&self) -> &'static str {
        self.vendor
    }
}

// This is the main window that we will use to render the game
pub struct Window {
    glutin: glutin::window::Window,
    device: Device,
}

impl Window {
    // Create a new window wrapper using a Glutin window
    pub(crate) fn new(_ctx: &mut Context, glutin: glutin::window::Window) -> Self {
        let _size = vek::Extent2::<u32>::from(<(u32, u32)>::from(glutin.inner_size())).as_::<u16>();

        Self {
            glutin,
            device: unsafe {
                Device {
                    name: get_static_str(gl::RENDERER),
                    vendor: get_static_str(gl::VENDOR),
                }
            },
        }
    }

    // Get the raw glutin window
    pub fn raw(&self) -> &glutin::window::Window {
        &self.glutin
    }

    // Get the window's device (OpenGL software/hardware that will be responsible for rendering onto this window)
    pub fn device(&self) -> &Device {
        &self.device
    }

    // Clear the whole window using the proper flags and values
    pub fn clear(
        &mut self,
        color: Option<vek::Rgb<f32>>,
        depth: Option<f32>,
        stencil: Option<i32>,
    ) {
        let mut flags = 0u32;

        // Set the background color values
        if let Some(color) = color {
            unsafe {
                gl::ClearColor(color.r, color.g, color.b, 1.0);
                flags |= gl::COLOR_BUFFER_BIT
            }
        }

        // Set the background depth values
        if let Some(depth) = depth {
            unsafe {
                gl::ClearDepth(depth as f64);
                flags |= gl::DEPTH_BUFFER_BIT;
            }
        }

        // Set the background stencil values
        if let Some(stencil) = stencil {
            unsafe {
                gl::ClearStencil(stencil);
                flags |= gl::STENCIL_BUFFER_BIT;
            }
        }

        // Clear the whole canvas using the proper bitwise flags
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
            gl::Clear(flags);
        }
    }
}

impl Display for Window {
    fn name(&self) -> u32 {
        0
    }

    fn writable_attachments_mask(&self) -> u32 {
        1 | (1 << 30)
    }

    fn viewport(&self) -> Viewport {
        let size = self.glutin.inner_size().cast::<u16>();
        Viewport {
            origin: vek::Vec2::zero(),
            extent: vek::Extent2::new(size.width, size.height),
        }
    }
}
