// A device is just some sort of wrapper around glutin windows
pub struct Device {
    // The underlying glutin window
    glutin: glutin::window::Window,

    // Size (in points) of the device window
    size: vek::Extent2<u32>,

    // Fullscreen state of the window
    fullscreen: bool,
} 

impl Device {
    // Create a new window using a Glutin window
    pub(crate) fn new(glutin: glutin::window::Window) -> Self {
        // Convert the size into a tuple
        let size = vek::Extent2::from(<(u32, u32)>::from(glutin.inner_size()));
        let fullscreen = glutin.fullscreen().is_some();

        Self { glutin, size, fullscreen }
    }
}