use ecs::Component;
use utils::Handle;
use crate::{Material, MaterialId, Mesh};

// A surface is a combination of a sub mesh and a specific material handle
// A renderable entity can have multiple surfaces that each have their own material
#[derive(Component)]
pub struct Surface<M: Material> {
    // Graphic object handles
    pub mesh: Handle<Mesh>,
    pub material: Handle<M>,
    
    // Surface settings
    pub visible: bool,
    pub culled: bool,

    // TODO: Figure out culling bounds maybe?

    // This does nothing and it has a size of 0, but let's keep it for clarity
    pub id: MaterialId<M>,
}

impl<M: Material> Surface<M> {
    // Create a new visible surface from a mesh handle, material handle, and material ID
    pub fn new(
        mesh: Handle<Mesh>,
        material: Handle<M>,
        id: MaterialId<M>,
    ) -> Self {
        Self {
            mesh,
            material,
            visible: true,
            culled: false,
            id,
        }
    }
}
