use super::SubMesh;
use crate::{context::Graphics, material::Material, shader::Shader};
use ecs::{Component, EcsManager};
use math::Transform;
use world::resources::{Handle, Storage};

// A surface is a combination of a sub mesh and a specific material handle
// A renderable entity will have multiple surface sets
#[derive(Component)]
pub struct Surface<M: Material>(Handle<SubMesh>, Handle<M>);

impl<M: Material> Surface<M> {
    // Create a new surface using a material handle and a submesh handle
    pub fn new(submesh: Handle<SubMesh>, material: Handle<M>) -> Self {
        Self(submesh, material)
    }

    // Get the submesh handle
    pub fn submesh(&self) -> &Handle<SubMesh> {
        &self.0
    }

    // Get the material handle
    pub fn material(&self) -> &Handle<M> {
        &self.1
    }
}
