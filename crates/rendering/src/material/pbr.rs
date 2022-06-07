use world::resources::Handle;

use crate::{
    shader::Uniforms,
    texture::{Ranged, Texture2D, R, RGB},
};

// Type aliases for texture maps
type DiffuseMap = Texture2D<RGB<Ranged<u8>>>;
type NormalMap = Texture2D<RGB<Ranged<u8>>>;
type MaskMap = Texture2D<RGB<Ranged<u8>>>;

// A physically based material that will try to replicate the behavior of real light
pub struct StandardMaterial {
    // Texture maps used for rendering
    diffuse: Option<Handle<DiffuseMap>>,
    normal: Option<Handle<NormalMap>>,
    mask: Option<Handle<MaskMap>>,

    // Texture parameters
    bumpiness: f32,
    roughness: f32,
    metallic: f32,
}

impl Default for StandardMaterial {
    fn default() -> Self {
        Self {
            diffuse: None,
            normal: None,
            mask: None,
            
            bumpiness: 1.0,
            roughness: 1.0,
            metallic: 1.0,
        }
    }
}
