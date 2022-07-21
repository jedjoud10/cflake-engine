use crate::buffer::BufferMode;
use math::AABB;

// Mesh settings that we will use whenever we import a new mesh from a file
#[derive(Clone, Copy)]
pub struct MeshImportSettings {
    pub mode: BufferMode,
    pub generate_normals: bool,
    pub generate_tangents: bool,
    pub scale: f32,
}

impl Default for MeshImportSettings {
    fn default() -> Self {
        Self {
            mode: BufferMode::Dynamic,
            generate_normals: false,
            generate_tangents: true,
            scale: 1.0,
        }
    }
}
