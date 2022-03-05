use crate::{pipeline::Handle, basics::texture::Texture};

// Material textures
#[derive(Default)]
pub struct MaterialTextures {
    pub diffuse_map: Handle<Texture>,
    pub normal_map: Handle<Texture>,
    pub emissive_map: Handle<Texture>,
}