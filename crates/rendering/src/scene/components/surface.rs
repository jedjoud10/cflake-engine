use crate::{Material, MaterialId, Mesh};
use ecs::Component;
use graphics::{DrawIndexedIndirectBuffer, DrawIndirectBuffer};
use utils::Handle;

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
    pub bounds: Option<math::Aabb<f32>>,

    // Indirect draw buffer that we can use to render this surface
    pub indirect: Option<Handle<DrawIndexedIndirectBuffer>>,

    // Needed to force the user to initialize the material
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
            indirect: None,
            bounds: None,
            culled: false,
            id,
        }
    }

    // Create a new visible indirect rendering surface
    pub fn indirect(
        mesh: Handle<Mesh>,
        material: Handle<M>,
        indirect: Handle<DrawIndexedIndirectBuffer>,
        id: MaterialId<M>,
    ) -> Self {
        Self {
            mesh,
            material,
            visible: true,
            culled: false,
            indirect: Some(indirect),
            bounds: None,
            id,
        }
    }
}
