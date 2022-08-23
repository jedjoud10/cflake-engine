use super::{get_static_str, Context};
use crate::{display::{ScopedCanvas, Display}, prelude::Viewport};

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
    pub(crate) fn new(ctx: &mut Context, glutin: glutin::window::Window) -> Self {
        let size = vek::Extent2::<u32>::from(<(u32, u32)>::from(glutin.inner_size())).as_::<u16>();

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
}

impl Display for Window {
    fn name(&self) -> u32 {
        0
    }

    fn viewport(&self) -> Viewport {
        let size = self.glutin.inner_size().cast::<u16>();
        Viewport {
            origin: vek::Vec2::zero(),
            extent: vek::Extent2::new(size.width, size.height),
        }
    }
}
