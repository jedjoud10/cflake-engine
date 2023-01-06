use graphics::{BufferUsage, BufferMode};
use crate::attributes::TexCoord;

// Mesh settings that we will use whenever we import a new mesh from a file
#[derive(Clone, Copy, PartialEq)]
pub struct MeshImportSettings {
    // How the mesh attribute buffers shall be created
    pub buffer_mode: BufferMode,
    pub buffer_usage: BufferUsage,

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
            buffer_mode: BufferMode::Resizable,
            buffer_usage: BufferUsage::CpuToGpu,
            invert_triangle_ordering: false,
            translation: vek::Vec3::zero(),
            rotation: vek::Quaternion::identity(),
            scale: vek::Vec3::one(),
        }
    }
}