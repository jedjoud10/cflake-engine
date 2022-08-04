use crate::buffer::BufferMode;
use math::AABB;

// Mesh settings that we will use whenever we import a new mesh from a file
#[derive(Clone, Copy)]
pub struct MeshImportSettings {
    pub mode: BufferMode,
    pub generate_normals: bool,
    pub generate_tangents: bool,
    pub position_scale: f32,
    pub invert_triangle_ordering: bool,
    pub invert_normals: bool,
    pub invert_tangents: bool,
    pub invert_vertical_tex_coord: bool,
    pub invert_horizontal_tex_coord: bool,
}

impl Default for MeshImportSettings {
    fn default() -> Self {
        Self {
            mode: BufferMode::Resizable,
            generate_normals: false,
            generate_tangents: true,
            position_scale: 1.0,
            invert_triangle_ordering: false,
            invert_normals: false,
            invert_tangents: false,
            invert_vertical_tex_coord: false,
            invert_horizontal_tex_coord: false,
        }
    }
}
