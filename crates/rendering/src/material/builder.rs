use std::marker::PhantomData;

use assets::Assets;
use world::Storage;

use crate::{context::Context, prelude::Shader};

use super::Material;

// Instance builder that will take a unique material and construct a new instance for it
// You can implement the instance builder for your specic material to write some methods that use the builder pattern
pub struct MaterialBuilder<M: Material>(M);

impl<M: Material> Default for MaterialBuilder<M> {
    // Create a default instance builder by creating a new material from it's instance
    fn default() -> Self {
        Self(M::default(InstanceID(Default::default())))
    }
}

impl<M: Material> MaterialBuilder<M> {
    // Get an immutable reference to the underlying material
    pub fn material(&self) -> &M {
        &self.0
    }

    // Get a mutable reference to the underlying material
    pub fn material_mut(&mut self) -> &mut M {
        &mut self.0
    }

    // Build the underlying material instance
    pub fn build(self, ctx: &mut Context, loader: &mut Assets, storage: &mut Storage<Shader>) -> M {
        // Add the material type renderer to the context
        ctx.register_material_renderer::<M, _>(|ctx| M::renderer(ctx, loader, storage));

        // Simply return the built material
        self.0
    }
}

// This is an Instance ID that will be stored within the materials
// By itself it does nothing, but we have to store it since the only way we can generate an instance ID is by using the descriptor
pub struct InstanceID<M: Material>(PhantomData<M>);
