use crate::Context;

// A window that we will use to render the game
pub struct Window {
    glutin: glutin::window::Window,
    size: vek::Extent2<u32>,
    fullscreen: bool,
}

impl Window {
    // Create a new window using a Glutin window
    pub(crate) fn new(glutin: glutin::window::Window) -> Self {
        // Convert the size into a tuple
        let size = vek::Extent2::from(<(u32, u32)>::from(glutin.inner_size()));
        let fullscreen = glutin.fullscreen().is_some();

        Self { glutin, size, fullscreen }
    }
}
