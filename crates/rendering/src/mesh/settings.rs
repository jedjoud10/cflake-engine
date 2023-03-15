use graphics::{BufferMode, BufferUsage};

// Mesh settings that we will use whenever we import a new mesh from a file
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeshImportSettings {
    // How the mesh attribute buffers shall be created
    pub buffer_mode: BufferMode,
    pub buffer_usage: BufferUsage,

    // Optional attributes we can discard when loading
    pub use_normals: bool,
    pub use_tangents: bool,
    pub use_tex_coords: bool,

    // We can invert all of the attributes if we want to
    pub invert_normals: bool,
    pub invert_tangents: bool,
    pub invert_tex_coords: vek::Vec2<bool>,

    // Invert the triangle ordering to make the mesh inside out
    pub invert_triangle_ordering: bool,

    // Custom transformations that we might want to apply to the mesh
    pub translation: vek::Vec3<f32>,
    pub rotation: vek::Quaternion<f32>,
    pub scale: vek::Vec3<f32>,
}

impl Default for MeshImportSettings {
    fn default() -> Self {
        Self {
            buffer_mode: BufferMode::default(),
            buffer_usage: BufferUsage::default(),
            use_normals: true,
            use_tangents: true,
            use_tex_coords: true,
            invert_normals: false,
            invert_tangents: false,
            invert_tex_coords: vek::Vec2::broadcast(false),
            invert_triangle_ordering: false,
            translation: vek::Vec3::zero(),
            rotation: vek::Quaternion::identity(),
            scale: vek::Vec3::one(),
        }
    }
}
