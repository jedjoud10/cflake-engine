use ecs::Component;
use utils::Handle;
use crate::Mesh;

// This is a culler component that can be added onto entities that have the renderer component
// This is implemented as an enum because we have the ability between letting the engine pick the bounds automatically or do them ourselves
#[derive(Component)]
pub struct Culler {
    // Culling params and bounds 
    pub culled: bool,
    pub aabb: math::Aabb<f32>,

    // Mesh that we can use to automatically get the bounds and use them
    pub mesh: Option<Handle<Mesh>>,
}

impl Culler {
    // Create a new culler based on the AABB of a mesh
    pub fn from_mesh(mesh: Handle<Mesh>) -> Self {
        Self {
            culled: false,
            aabb: math::Aabb::<f32> {
                min: vek::Vec3::zero(),
                max: vek::Vec3::zero()
            },
            mesh: Some(mesh)
        }
    }
}