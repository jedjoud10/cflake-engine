use math::AABB;
use crate::buffer::BufferMode;

// The mesh mode tells use what are the modes for each specific vertex attribute
#[derive(Clone, Copy)]
pub struct MeshImportMode {
    pub positions: BufferMode,
    pub normals: BufferMode,
    pub tangents: BufferMode,
    pub colors: BufferMode,
    pub tex_coords: BufferMode,
    pub indices: BufferMode,
}

impl MeshImportMode {
    // Static meshes are created only once and they cannot be modified later on 
    pub const Static: Self = Self {
        positions: BufferMode::Static,
        normals: BufferMode::Static,
        tangents: BufferMode::Static,
        colors: BufferMode::Static,
        tex_coords: BufferMode::Static,
        indices: BufferMode::Static,
    };

    // Dynamic meshes can be modified by changing each attribute's properties, but the number of vertices stays constant
    pub const Dynamic: Self = Self {
        positions: BufferMode::Dynamic,
        normals: BufferMode::Dynamic,
        tangents: BufferMode::Dynamic,
        colors: BufferMode::Dynamic,
        tex_coords: BufferMode::Dynamic,
        indices: BufferMode::Dynamic,
    };

    // Procedural meshes can change the number of vertices and triangles from within them
    pub const Procedural: Self = Self {
        positions: BufferMode::Resizable,
        normals: BufferMode::Resizable,
        tangents: BufferMode::Resizable,
        colors: BufferMode::Resizable,
        tex_coords: BufferMode::Resizable,
        indices: BufferMode::Resizable,
    };
}

// Mesh settings that we will use whenever we import a new mesh from a file
#[derive(Clone, Copy)]
pub struct MeshImportSettings {
    pub mode: MeshImportMode,
    pub generate_tangents: bool,
    pub scale: f32,
}

impl Default for MeshImportSettings {
    fn default() -> Self {
        Self { 
            mode: MeshImportMode::Dynamic,
            generate_tangents: true,
            scale: 1.0
        }
    }
}