use assets::loader::AssetLoader;
use world::resources::{Handle, Storage};

use crate::{
    context::{Context, Device},
    mesh::SubMesh,
    shader::{Shader, Uniforms},
};

// A material descriptor helps us create materials by setting their properties and fields through the builder pattern
pub trait Descriptor<'ctx, 'c, M: Material>: Into<M> {
    // Create a new builder that will store the context internally
    fn new(ctx: &'ctx mut Context, storages: &'c mut Storage<M>) -> Self;

    // Get a mutable reference to the underlying context
    fn ctx_mut(&mut self) -> &'ctx mut Context;

    // Get a mutable reference to the material storage
    fn storage_mut(&mut self) -> &'c mut Storage<M>;

    // Construct the material from the descriptor
    fn build() -> Handle<M> {
        // Insert the material renderer into the context, and store the actual material into the storage

    }
}

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
pub trait Material: Sized {
    // Load the corresponding default shader for this material
    fn shader(loader: &mut AssetLoader) -> Shader;

    // Set the corresponding uniforms for this material
    fn set(uniforms: &mut Uniforms);

    // Render all the objects that use this shared material
    fn render(&self, ecs: &mut ecs::EcsManager) {
        
    }
}
