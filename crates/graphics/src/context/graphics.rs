use super::Window;

use world::Resource;

// Graphical settings that we will use to create the graphical context
#[derive(Clone)]
pub struct GraphicsSettings {}

// Graphical context that we will wrap around the Vulkan instance
// This will also wrap the logical device that we will select
// This context is shareable between threads because Vulkan is multithreaded
#[derive(Clone)]
pub struct Graphics {    
}

impl Graphics {
    // Create a new graphics context based on the window wrapper
    pub(crate) fn new(window: &Window, settings: GraphicsSettings) -> Graphics {
        Graphics {}
    }
}
