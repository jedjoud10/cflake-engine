use assets::loader::AssetLoader;

use crate::{context::{Context, Device}, shader::{Shader, Uniforms}, mesh::SubMesh};

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
pub trait Material {
    // Get the execution order for this material
    fn layer() -> i32;

    // Create this new material by initializing it with the default shader 
    fn new(ctx: &mut Context, loader: &mut AssetLoader) -> Self;

    // Create this new material using an explicit shader
    fn with_shader(ctx: &mut Context, shader: Shader) -> Self;

    // Set the shader uniforms and return the shader
    fn set_uniforms(&mut self, ctx: &mut Context, device: &mut Device) -> &Shader;

    // Draw all of the given submeshes
    fn execute(&self, to_draw: Vec<&dyn Draw>) -> ;
}