use crate::{
    Direct, Indirect, IndirectMesh, Material, MaterialId, Mesh,
};
use ecs::Component;

use utils::Handle;

// A surface is a combination of a sub mesh and a specific material handle
// A renderable entity can have multiple surfaces that each have their own material
#[derive(Component)]
pub struct Surface<M: Material> {
    // Graphic object handles
    pub mesh: Handle<Mesh<M::RenderPath>>,
    pub material: Handle<M>,

    // Surface settings
    pub visible: bool,
    pub culled: bool,

    // Shadow parameters
    pub shadow_caster: bool,
    pub shadow_receiver: bool,
    pub shadow_culled: bool,

    // Needed to force the user to initialize the material
    pub id: MaterialId<M>,
}

impl<M: Material<RenderPath = Direct>> Surface<M> {
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
            shadow_caster: true,
            shadow_receiver: true,
            shadow_culled: false,
        }
    }
}

impl<M: Material<RenderPath = Indirect>> Surface<M> {
    // Create a new visible surface from a mesh handle, material handle, and material ID
    pub fn indirect(
        mesh: Handle<IndirectMesh>,
        material: Handle<M>,
        id: MaterialId<M>,
    ) -> Self {
        Self {
            mesh,
            material,
            visible: true,
            culled: false,
            id,
            shadow_caster: true,
            shadow_receiver: true,
            shadow_culled: false,
        }
    }
}
