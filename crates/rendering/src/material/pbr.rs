use world::resources::Handle;

use crate::{
    shader::{Uniforms, Matrix},
    texture::{Ranged, Texture2D, R, RGB},
};

use super::Material;

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

impl Material for StandardMaterial {
    fn load_shader(loader: &mut assets::loader::AssetLoader) -> crate::shader::Shader {
        todo!()
    }

    fn with_shader(ctx: &mut crate::context::Context, shader: crate::shader::Shader) -> Self {
        todo!()
    }

    fn set_uniforms(&mut self, ctx: &mut crate::context::Context, device: &mut crate::context::Device) {
        todo!()
    }

    fn shader(&self) -> &crate::shader::Shader {
        todo!()
    }
}