use assets::loader::AssetLoader;

use crate::{shader::{Shader, Uniforms}, context::Context};

// A material is what we shall use to render surfaces onto the screen
pub trait Material {
    // Load the shader that we will use for this material
    fn shader(ctx: &mut Context, loader: &mut AssetLoader) -> Shader;
}

// A material instance is just like a material, but it will actually set the uniforms directly
pub trait MaterialInstance {
    // Set the valid uniforms from this material
   // fn set(&mut self, storage: &mut Storage, uniforms: Uniforms);
}