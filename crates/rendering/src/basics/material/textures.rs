use crate::{basics::texture::{Texture, Texture2D}, pipeline::Handle};

// Material textures
#[derive(Default)]
pub struct MaterialTextures {
    pub diffuse_map: Handle<Texture2D>,
    pub normal_map: Handle<Texture>,
    pub emissive_map: Handle<Texture>,
}
