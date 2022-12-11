use std::sync::Arc;

// Internal states that contain the raw vulkan instances and values
struct Raw {}

// Graphical context that we will wrap around the Vulkan instance
// This context must be shareable between threads to allow for multithreading
#[derive(Clone)]
pub struct Graphics(Arc<Raw>);
