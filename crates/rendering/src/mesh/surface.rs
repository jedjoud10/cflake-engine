use super::Mesh;
use crate::material::Material;
use crate::pipeline::{PipeId, SpecializedPipeline};
use ecs::Component;

use math::AABB;
use world::Handle;

// A surface is a combination of a sub mesh and a specific material handle
// A renderable entity will have multiple surface sets
#[derive(Component)]
pub struct Surface<M: for<'w> Material<'w>> {
    // Graphic object handles
    mesh: Handle<Mesh>,
    material: Handle<M>,

    // This does nothing and it has a size of 0, but let's keep it for clarity
    id: PipeId<SpecializedPipeline<M>>,
}

impl<M: for<'w> Material<'w>> Surface<M> {
    // Create a new surface that can be rendered
    pub fn new(
        mesh: Handle<Mesh>,
        material: Handle<M>,
        id: PipeId<SpecializedPipeline<M>>,
    ) -> Self {
        Self {
            mesh,
            material,
            id,
        }
    }

    // Get the mesh handle
    pub fn mesh(&self) -> Handle<Mesh> {
        self.mesh.clone()
    }

    // Get the material handle
    pub fn material(&self) -> Handle<M> {
        self.material.clone()
    }
}
