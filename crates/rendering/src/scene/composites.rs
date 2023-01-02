use graphics::{Texture, BGRA, Normalized, Texture2D};

// Main resource that will contain data to render objects on the screen
// This will contain the current swapchain texture that we must render to
pub struct ForwardRenderer {
    pub(crate) frame: Texture2D<BGRA<Normalized<u8>>>,
}