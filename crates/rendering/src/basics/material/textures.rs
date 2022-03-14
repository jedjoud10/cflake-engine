use crate::{
    basics::texture::{BundledTexture2D, Texture, Texture2D, TextureHandle},
    pipeline::Handle,
};
// Material textures
#[derive(Default)]
pub struct MaterialTextures {
    pub diffuse_map: TextureHandle,
    pub normal_map: TextureHandle,
    pub emissive_map: TextureHandle,
}
