use crate::{
    Direct, Indirect, IndirectMesh, Material, MaterialId, Mesh, RenderPath,
};
use ecs::Component;
use smallvec::SmallVec;
use utils::Handle;

// A surface is a combination of multiple meshes and a specific material handle
// A renderable entity can have multiple surfaces that each have their own different material type
#[derive(Component)]
pub struct Surface<M: Material> {
    // Graphic object handles
    pub meshes: SmallVec<[Handle<Mesh<M::RenderPath>>; 1]>,
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

impl<M: Material> Surface<M> {
    // Create a new visible surface from a mesh handle, material handle, and material ID
    pub fn new(
        mesh: Handle<Mesh<M::RenderPath>>,
        material: Handle<M>,
        id: MaterialId<M>,
    ) -> Self {
        Self {
            meshes: SmallVec::from_buf([mesh]),
            material,
            visible: true,
            culled: false,
            id,
            shadow_caster: true,
            shadow_receiver: true,
            shadow_culled: false,
        }
    }

    // Add a new mesh to the list of meshes to render using this material
    pub fn push(&mut self, mesh: Handle<Mesh<M::RenderPath>>,) {
        self.meshes.push(mesh);
    }

    // List the internally stored meshes immutably
    pub fn meshes(&self) -> &[Handle<Mesh<M::RenderPath>>] {
        &self.meshes
    }
    
    // List the internally stored meshes mutably
    pub fn meshes_mut(&mut self) -> &mut [Handle<Mesh<M::RenderPath>>] {
        self.meshes.as_mut_slice()
    }
}