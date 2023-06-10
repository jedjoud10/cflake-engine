use crate::{Material, MaterialId, Mesh};
use ecs::Component;
use smallvec::SmallVec;
use utils::Handle;

// A sub surface is a combination of a mesh and a material
// We can store multiple sub-surfaces into a surface to create multi-material systems
pub struct SubSurface<M: Material> {
    pub mesh: Handle<Mesh<M::RenderPath>>,
    pub material: Handle<M>,
}

impl<M: Material> Clone for SubSurface<M> {
    fn clone(&self) -> Self {
        Self {
            mesh: self.mesh.clone(),
            material: self.material.clone(),
        }
    }
}

// A surface is a combination of multiple subsurfaces to create a whole "mesh cluster" that a material can render
// A renderable entity can have multiple surfaces that each have their own different material type
#[derive(Component)]
pub struct Surface<M: Material> {
    // I LOVE SUBSURFACES
    pub subsurfaces: SmallVec<[SubSurface<M>; 1]>,

    // Surface settings
    pub visible: bool,
    pub culled: bool,

    // Shadow parameters
    pub shadow_caster: bool,
    pub shadow_culled: bool,

    // Needed to force the user to initialize the material
    pub id: MaterialId<M>,
}

impl<M: Material> Clone for Surface<M> {
    fn clone(&self) -> Self {
        Self {
            subsurfaces: self.subsurfaces.clone(),
            visible: self.visible,
            culled: self.culled,
            shadow_caster: self.shadow_caster,
            shadow_culled: self.shadow_culled,
            id: self.id.clone(),
        }
    }
}

impl<M: Material> Surface<M> {
    // Create a new visible surface from a mesh handle, material handle, and material ID
    pub fn new(mesh: Handle<Mesh<M::RenderPath>>, material: Handle<M>, id: MaterialId<M>) -> Self {
        Self {
            subsurfaces: SmallVec::from_buf([SubSurface { mesh, material }]),
            visible: true,
            culled: false,
            id,
            shadow_caster: true,
            shadow_culled: false,
        }
    }
}
