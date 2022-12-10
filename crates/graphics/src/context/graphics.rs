// Graphical context that we will wrap around the WGPU instance
// This context must be shareable between threads to allow for multithreading
#[derive(Clone)]
pub struct Graphics {}
