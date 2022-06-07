use world::resources::Handle;

use crate::{
    shader::{Uniforms, Matrix},
    texture::{Ranged, Texture2D, R, RGB},
};

use super::{Material, Descriptor};

// Type aliases for texture maps
type DiffuseMap = Texture2D<RGB<Ranged<u8>>>;
type NormalMap = Texture2D<RGB<Ranged<u8>>>;
type MaskMap = Texture2D<RGB<Ranged<u8>>>;

// A physically based descriptor that will try to replicate the behavior of real lighting
pub struct PhysicallyBased {
    // Texture maps used for rendering
    diffuse: Option<Handle<DiffuseMap>>,
    normal: Option<Handle<NormalMap>>,
    mask: Option<Handle<MaskMap>>,

    // Texture parameters
    bumpiness: f32,
    roughness: f32,
    metallic: f32,
}

impl PhysicallyBased {
    // Set the diffuse texture
    pub fn diffuse(mut self, diffuse: Handle<DiffuseMap>) -> Self {
        self.diffuse = Some(diffuse);
        self
    }

    // Set the normal texture
    pub fn normal(mut self, normal: Handle<NormalMap>) -> Self {
        self.normal = Some(normal);
        self
    }

    // Set the mask texture
    pub fn mask(mut self, diffuse: Handle<MaskMap>) -> Self {
        self.diffuse = Some(diffuse);
        self
    }

    // Set the bumpiness parameter
    pub fn bumpiness(mut self, bumpiness: f32) -> Self {
        self.bumpiness = bumpiness;
        self
    }

    // Set the roughness parameter
    pub fn roughness(mut self, roughness: f32) -> Self {
        self.roughness = roughness;
        self
    }

    // Set the metallic parameter
    pub fn metallic(mut self, metallic: f32) -> Self {
        self.metallic = metallic;
        self
    }
}

impl Default for PhysicallyBased {
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

impl Descriptor for PhysicallyBased {
    fn shader(&self) -> &crate::shader::Shader {
        todo!()
    }

    fn to_material(self) -> Material {
        todo!()
    }
}