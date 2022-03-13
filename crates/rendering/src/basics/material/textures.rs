use crate::{basics::texture::{Texture, Texture2D, BundledTexture2D, TextureHandle}, pipeline::Handle};
// Material textures
#[derive(Default)]
pub struct MaterialTextures {
    pub diffuse_map: TextureHandle,
    pub normal_map: TextureHandle,
    pub emissive_map: TextureHandle,
}
