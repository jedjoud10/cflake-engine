use bitflags::bitflags;

use super::Vertices;
bitflags! {
    pub struct MeshFlags: u8 {
        const TANGENTS_SUPPORTED = 1;
        const NORMALS_SUPPORTED = 1 << 1;
        const COLORS_SUPPORTED = 1 << 2;
        const UVS_SUPPORTED = 1 << 3;
    }
}

impl MeshFlags {
    // Calculate the mesh flags from the given vertices
    pub fn get(vertices: &Vertices) -> Self {
        Self::empty()
    }
}