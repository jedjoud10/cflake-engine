use super::Mesh;
use crate::material::{Material, MaterialId};
use ecs::Component;

use world::Handle;

// A surface is a combination of a sub mesh and a specific material handle
// A renderable entity will have multiple surface sets
#[derive(Component)]
pub struct Surface<M: for<'w> Material<'w>> {
    // Graphic object handles
    pub mesh: Handle<Mesh>,
    pub material: Handle<M>,

    // Surface settings
    pub visible: bool,
    pub shadow_caster: bool,
    pub shadow_receiver: bool,

    // This does nothing and it has a size of 0, but let's keep it for clarity
    pub matid: MaterialId<M>,
}

impl<M: for<'w> Material<'w>> Surface<M> {
    pub fn new(mesh: Handle<Mesh>, material: Handle<M>, id: MaterialId<M>) -> Self {
        Self {
            mesh,
            material,
            visible: true,
            shadow_caster: true,
            shadow_receiver: true,
            matid: id,
        }
    }
}
