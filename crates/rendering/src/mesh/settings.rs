use crate::buffer::BufferMode;
use math::AABB;

// Mesh settings that we will use whenever we import a new mesh from a file
#[derive(Clone, Copy)]
pub struct MeshImportSettings {
    pub mode: BufferMode,
    pub generate_normals: bool,
    pub generate_tangents: bool,
    pub position_scale: f32,
    pub invert_normal: bool,
    pub invert_tangents: bool,
    pub invert_vertical_uv: bool,
    pub invert_horizontal_uv: bool,
}

impl Default for MeshImportSettings {
    fn default() -> Self {
        Self {
            mode: BufferMode::Dynamic,
            generate_normals: false,
            generate_tangents: true,
            position_scale: 1.0,
            invert_normal: false,
            invert_tangents: false,
            invert_vertical_uv: false,
            invert_horizontal_uv: false,
        }
    }
}
