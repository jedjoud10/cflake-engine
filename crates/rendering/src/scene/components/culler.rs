use ecs::Component;


// This is a culler component that can be added onto entities that have the renderer component
#[derive(Component)]
pub struct Culler {
    // Is the entity currently culled?
    pub culled: bool,

    // Culling AABB of the entities
    pub aabb: vek::Aabb<f32>
}

impl Culler {
    // Create a new culler based on the AABB of a mesh
    pub fn from_mesh(mesh: &crate::Mesh) -> Self {
        Self {
            culled: false,
            aabb: mesh.vertices().aabb().unwrap(),
        }
    }
}