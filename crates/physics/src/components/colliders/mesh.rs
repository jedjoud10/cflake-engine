use rendering::Mesh;
use utils::Handle;

// Mesh collider that will represent a mesh using it's triangles and vertices
// You can only use the mesh of a direct mesh, since we do not know the mesh of indirectly rendered entities
pub enum MeshCollider {
    // Discrete mesh colliders expect you to set their triangles and vertices manually
    Explicity {
        vertices: Handle<Vec<vek::Vec3<f32>>>,
        triangles: Handle<Vec<u32>>,
    },

    // Automatic mesh colliders fetch their mesh from the GPU buffers automatically
    // These are inherintely slower than explicit mesh colliders since you need to create the mesh on the GPU
    Fetched {
        mesh: Handle<Mesh>,
    }
}