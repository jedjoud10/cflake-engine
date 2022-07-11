use math::AABB;
use crate::buffer::BufferMode;

// The mesh mode tells use what are the modes for each specific vertex attribute
#[derive(Clone, Copy)]
pub enum MeshImportMode {
    // Static meshes are created only once and they cannot be modified later on 
    Static,
    
    // Dynamic meshes can be modified by changing each attribute's properties, but the number of vertices stays constant
    Dynamic,
    
    // Procedural meshes can change the number of vertices and triangles from within them
    Procedural,
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