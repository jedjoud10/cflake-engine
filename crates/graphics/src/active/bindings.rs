use crate::context::Graphics;

// Wrapper around a WGPU bind group. I wrap this myself so I can pass my own bind group wrappers like the texture/buffer wrappers directly
pub struct BindGroup(wgpu::BindGroup);

impl BindGroup {
    // Create a new bind group from the given resources for a specific shader type
    // Can receive both compute / render shader bundles and a specific "ID" to specific what bind group layout to use
    pub fn new(graphics: &Graphics) -> Self {
        todo!()
    }
}