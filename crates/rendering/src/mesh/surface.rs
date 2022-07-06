use super::SubMesh;
use crate::{canvas::FaceCullMode, material::Material};
use ecs::Component;

use math::AABB;
use world::Handle;

// A surface is a combination of a sub mesh and a specific material handle
// A renderable entity will have multiple surface sets
#[derive(Component)]
pub struct Surface<M: for<'w> Material<'w>> {
    // Graphic object handles
    submesh: Handle<SubMesh>,
    material: Handle<M>,

    // Bounds of the surface
    bounds: Option<AABB>,
}

impl<M: for<'w> Material<'w>> Surface<M> {
    // Create a new surface that can be rendered
    pub fn new(submesh: Handle<SubMesh>, material: Handle<M>) -> Self {
        Self {
            submesh,
            material,
            bounds: None,
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

    // Get the submesh handle
    pub fn submesh(&self) -> Handle<SubMesh> {
        self.submesh.clone()
    }

    // Get the material handle
    pub fn material(&self) -> Handle<M> {
        self.material.clone()
    }
}
