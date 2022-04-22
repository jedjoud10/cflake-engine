use bitflags::bitflags;

use super::Vertices;
bitflags! {
    pub struct MeshFlags: u8 {
        const TANGENTS_SUPPORTED = 1;
        const NORMALS_SUPPORTED = 1 << 1;
        const VERTEX_COLORS_SUPPORTED = 1 << 2;
        const UVS_SUPPORTED = 1 << 3;
    }
}

impl MeshFlags {
    // Calculate the mesh flags from the given vertices
    pub fn get(vertices: &Vertices) -> Self {
        let mut me = Self::empty();
        // Detect normals
        if !vertices.normals.is_empty() {
            me.insert(Self::NORMALS_SUPPORTED);
        }

        // Detect tangents
        if !vertices.tangents.is_empty() {
            me.insert(Self::TANGENTS_SUPPORTED);
        }

        // Detect colors
        if !vertices.colors.is_empty() {
            me.insert(Self::VERTEX_COLORS_SUPPORTED);
        }

        // Detect uvs
        if !vertices.uvs.is_empty() {
            me.insert(Self::UVS_SUPPORTED);
        }

        me
    }
}
