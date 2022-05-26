use crate::{texture::{RGB, Ranged, Texture2D, R}, shader::Uniforms};

use super::{Material, MaterialInstance};

// Type aliases for texture maps
type DiffuseMap = Texture2D<RGB<Ranged<u8>>>;
type NormalMap = Texture2D<RGB<Ranged<u8>>>;
type MaskMap = Texture2D<RGB<Ranged<u8>>>;

// A physically based material that will try to replicate the behavior of real light
pub struct StandardMaterial {
    // Texture maps used for rendering
    diffuse: Option<DiffuseMap>,
    normal: Option<NormalMap>,
    mask: Option<MaskMap>,

    // Texture parameters
    normal_map_strength: f32,
    roughness_strength: f32,
    metallic_strength: f32,
}

impl Default for StandardMaterial {
    fn default() -> Self {
        Self { 
            diffuse: None,
            normal: None,
            mask: None, 
            normal_map_strength: 1.0,
            roughness_strength: 1.0,
            metallic_strength: 1.0 }
    }
}

impl Material for StandardMaterial {
    // This will load the default PBR shader
    fn shader(ctx: &mut crate::context::Context, loader: &mut assets::loader::AssetLoader) -> crate::shader::Shader {
        todo!()
    }
}

impl MaterialInstance for StandardMaterial {
    // Set the PBR uniforms
    /*
    fn set(&mut self, storage: &CacheStorage, uniforms: Uniforms) {
        uniforms.set_sampler("diffuse_map", self.)
    }
    */
}