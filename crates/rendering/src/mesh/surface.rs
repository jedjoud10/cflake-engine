use super::Mesh;
use crate::material::{Material};
use crate::pipeline::{PipeId, Pipeline, SpecializedPipeline};
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

    // Bounds of the surface
    bounds: Option<AABB>,

    // This does nothing and it has a size of 0, but let's keep it for clarity
    id: PipeId<SpecializedPipeline<M>>,
}

impl<M: for<'w> Material<'w>> Surface<M> {
    // Create a new surface that can be rendered
    pub fn new(mesh: Handle<Mesh>, material: Handle<M>, id: PipeId<SpecializedPipeline<M>>) -> Self {
        Self {
            mesh,
            material,
            bounds: None,
            id,
        }
    }

    // Set the AABB bounds of this surface manually
    pub fn set_aabb(&mut self, aabb: AABB) {
        self.bounds = Some(aabb);
    }

    // Get the AABB bounds
    pub fn aabb(&self) -> &Option<AABB> {
        &self.bounds
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
