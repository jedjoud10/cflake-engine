use crate::{Material, MaterialId, Mesh, CullResult};
use ecs::Component;
use smallvec::SmallVec;
use utils::Handle;

// A sub surface is a combination of a mesh and a material
// We can store multiple sub-surfaces into a surface to create multi-material systems
pub struct SubSurface<M: Material> {
    pub mesh: Handle<Mesh<M::RenderPath>>,
    pub material: Handle<M>,
    pub culled: CullResult,

    // SubSurface settings
    pub visible: bool,
    
    // If an object is completely within the frustum of the cascade then it is a waste to render it for the larger cascades as well
    pub shadow_culled: u8,

    // Shadow parameters
    pub shadow_caster: bool,
}

impl<M: Material> Clone for SubSurface<M> {
    fn clone(&self) -> Self {
        Self {
            mesh: self.mesh.clone(),
            material: self.material.clone(),
            culled: self.culled.clone(),
            visible: self.visible.clone(),
            shadow_culled: self.shadow_culled.clone(),
            shadow_caster: self.shadow_caster.clone(),
        }
    }
}

// A surface is a combination of multiple subsurfaces to create a whole "mesh cluster" that a material can render
// A renderable entity can have multiple surfaces that each have their own different material type
#[derive(Component)]
pub struct Surface<M: Material> {
    // I LOVE SUBSURFACES
    pub subsurfaces: SmallVec<[SubSurface<M>; 1]>,

    // Needed to force the user to initialize the material
    pub id: MaterialId<M>,
}

impl<M: Material> Clone for Surface<M> {
    fn clone(&self) -> Self {
        Self {
            subsurfaces: self.subsurfaces.clone(),
            id: self.id.clone(),
        }
    }
}

impl<M: Material> Surface<M> {
    // Create a new visible surface from a mesh handle, material handle, and material ID
    pub fn new(mesh: Handle<Mesh<M::RenderPath>>, material: Handle<M>, id: MaterialId<M>) -> Self {
        Self {
            subsurfaces: SmallVec::from_buf([SubSurface {
                mesh,
                material,
                culled: CullResult::Visible,
                visible: true,
                shadow_culled: 0,
                shadow_caster: true
            }]),
            id,
        }
    }

    // Create a new visible surface from multiple subsurfaces
    pub fn from_iter(
        subsurfaces: impl IntoIterator<Item = SubSurface<M>>,
        id: MaterialId<M>,
    ) -> Self {
        Self {
            subsurfaces: subsurfaces.into_iter().collect::<_>(),
            id,
        }
    }
}
