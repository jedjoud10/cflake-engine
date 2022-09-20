use crate::buffer::BufferMode;

// Mesh settings that we will use whenever we import a new mesh from a file
#[derive(Clone, Copy)]
pub struct MeshImportSettings {
    pub mode: BufferMode,
    pub use_normals: bool,
    pub use_tangents: bool,
    pub use_tex_coords: bool,
    pub invert_triangle_ordering: bool,
    pub invert_normals: bool,
    pub invert_tangents: bool,
    pub invert_tex_coords: vek::Vec2<bool>,
    pub translation: vek::Vec3<f32>,
    pub rotation: vek::Quaternion<f32>,
    pub scale: vek::Vec3<f32>,
}

impl Default for MeshImportSettings {
    fn default() -> Self {
        Self {
            mode: BufferMode::Dynamic {
                persistent: true,
                map_read: true,
                map_write: true,
                client: false,
            },
            use_normals: true,
            use_tangents: true,
            use_tex_coords: true,
            invert_triangle_ordering: false,
            invert_normals: false,
            invert_tangents: false,
            invert_tex_coords: vek::Vec2::new(false, false),
            translation: vek::Vec3::zero(),
            rotation: vek::Quaternion::identity(),
            scale: vek::Vec3::one(),
        }
    }
}
