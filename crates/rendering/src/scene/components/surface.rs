use ecs::Component;
use utils::Handle;

use crate::{Material, Mesh, MaterialId};

// A surface is a combination of a sub mesh and a specific material handle
// A renderable entity will have multiple surface sets
#[derive(Component)]
pub struct Surface<M: Material> {
    // Graphic object handles
    pub mesh: Handle<Mesh>,
    pub material: Handle<M>,

    // Surface settings
    pub visible: bool,

    // This does nothing and it has a size of 0, but let's keep it for clarity
    pub matid: MaterialId<M>,
}

impl<M: Material> Surface<M> {
    // Create a new visible surface from a mesh handle, material handle, and material ID
    pub fn new(mesh: Handle<Mesh>, material: Handle<M>, id: MaterialId<M>) -> Self {
        Self {
            mesh,
            material,
            visible: true,
            matid: id,
        }
    }
}
