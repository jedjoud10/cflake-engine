use assets::loader::AssetLoader;
use world::resources::{Handle, Storage};

use crate::{
    context::{Context, Device},
    mesh::SubMesh,
    shader::{Shader, Uniforms},
};

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
// Each material contains a properity block and a unique shader
// Currently, we cannot modify the values of materials after they have been created since it would require some boxing magic fuckery and such 
pub struct Material {
    shader: Shader,
}