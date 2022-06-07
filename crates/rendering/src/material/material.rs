use assets::loader::AssetLoader;

use crate::{
    context::{Context, Device},
    mesh::SubMesh,
    shader::{Shader, Uniforms},
};

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
pub struct Material {
    shader: Handle<Shader>,
}


// A material decriptor simply gives us the correct values to create a given material
pub trait Descriptor: Sized {
    // Get the underlying shader stored in the material
    fn shader(&self) -> &Shader;

    // Create a new generic material
    fn to_material(self) -> Material; 
}
