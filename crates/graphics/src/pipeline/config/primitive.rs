pub use wgpu::{FrontFace, Face};

// Depicts the exact primitives we will use to draw the VAOs
#[derive(Clone, Copy, PartialEq)]
pub enum PrimitiveConfig {
    // The pipeline will draw triangles onto the screen
    Triangles {
        winding_order: wgpu::FrontFace,
        cull_face: Option<wgpu::Face>,
        wireframe: bool,
    },

    // The pipeline will draw lines onto the screen
    Lines {
        width: f32,
    },

    // The pipeline will draw points onto the screen
    Points,
}