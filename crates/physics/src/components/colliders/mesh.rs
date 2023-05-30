use ecs::Component;
use rendering::Mesh;
use utils::Handle;

// Mesh collider variants since we can create =/fetch meshes in different ways
// You can only use the mesh of a direct mesh, since we do not know the mesh of indirectly rendered entities
pub(crate) enum InnerMeshCollider {
    ExplicitOwned {
        vertices: Vec<vek::Vec3<f32>>,
        triangles: Vec<[u32; 3]>,
    },

    /*
    Fetched {
        mesh: Handle<Mesh>,
    }
    */
}


// Mesh collider that will represent a mesh using it's triangles and vertices
#[derive(Component)]
pub struct MeshCollider {
    pub(crate) inner: Option<InnerMeshCollider>,
    pub mass: f32,
    pub friction: f32,
    pub restitution: f32,
    pub(crate) handle: Option<rapier3d::geometry::ColliderHandle>,
}

impl MeshCollider {
    // Create a new mesh collider with specific vertices and triangles
    pub fn new(vertices: Vec<vek::Vec3<f32>>, triangles: Vec<[u32; 3]>, mass: f32, friction: f32, restitution: f32) -> Self {
        Self {
            inner: Some(InnerMeshCollider::ExplicitOwned { vertices, triangles }),
            mass,
            friction,
            restitution,
            handle: None,
        }
    }
}