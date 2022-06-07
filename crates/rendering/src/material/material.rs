use assets::loader::AssetLoader;

use crate::{
    context::{Context, Device},
    mesh::SubMesh,
    shader::{Shader, Uniforms},
};

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
pub trait Material: Sized {
    // Get the default material shader that we might use
    fn load_shader(loader: &mut AssetLoader) -> Shader;

    // Create this new material by initializing it with it's default shader
    fn new(ctx: &mut Context, loader: &mut AssetLoader) -> Self {
        Self::with_shader(ctx, Self::load_shader(loader))
    }

    // Create this new material using an explicit shader
    fn with_shader(ctx: &mut Context, shader: Shader) -> Self;

    // Set the shader uniforms
    fn set_uniforms(&mut self, ctx: &mut Context, device: &mut Device);

    // Get the underlying shader stored in the material
    fn shader(&self) -> &Shader;
}
