use crate::vulkan::vk;

// How rasterized triangles should be culled
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FaceCullMode {
    Front(bool),
    Back(bool),
}

// Depicts the exact primitives we will use to draw the VAOs
#[derive(Clone, Copy, PartialEq)]
pub enum Primitive {
    Triangles {
        cull: Option<FaceCullMode>,
        wireframe: bool,
    },
    Lines {
        width: f32,
    },
    Points,
}