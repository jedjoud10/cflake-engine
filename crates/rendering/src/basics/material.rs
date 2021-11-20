use crate::basics::*;
use crate::pipeline::*;
use crate::utils::*;
use assets::{AssetManager, AssetObject, Object};
use bitflags::bitflags;
use std::sync::Arc;
use std::{collections::HashMap, rc::Rc};

bitflags! {
    pub struct MaterialFlags: u8 {
        const DOUBLE_SIDED = 0b00000001;
    }
}

// A material that can have multiple parameters and such
pub struct Material {
    // Rendering stuff
    pub shader: ShaderGPUObject,
    pub material_name: String,
    pub flags: MaterialFlags,
    pub uniforms: ShaderUniformsGroup,
    // The default textures
    pub diffuse_tex: TextureGPUObject,
    pub normal_tex: TextureGPUObject,
    // Is this material even visible?
    pub visible: bool,
}

impl Default for Material {
    fn default() -> Self {
        let mut material: Self = Material {
            shader: ShaderGPUObject::default(),
            material_name: String::new(),
            flags: MaterialFlags::empty(),
            uniforms: ShaderUniformsGroup::new_null(),
            diffuse_tex: TextureGPUObject::default(),
            normal_tex: TextureGPUObject::default(),
            visible: true,
        };
        // Set the default shader args
        material.uniforms.set_vec2f32("uv_scale", veclib::Vector2::ONE);
        material.uniforms.set_vec3f32("tint", veclib::Vector3::ONE);
        material.uniforms.set_f32("normals_strength", 1.0);
        material
    }
}

impl Material {
    // Create a new material with a name
    pub fn new(material_name: &str, asset_manager: &mut AssetManager) -> Self {
        Self {
            material_name: material_name.to_string(),
            diffuse_tex: pipec::texturec(Texture::object_load_o("white", &mut asset_manager.object_cacher)),
            normal_tex: pipec::texturec(Texture::object_load_o("default_normals", &mut asset_manager.object_cacher)),
            ..Self::default()
        }
    }
    // Load the diffuse texture
    pub fn load_diffuse(mut self, diffuse_path: &str, opt: Option<TextureLoadOptions>, asset_manager: &mut AssetManager) -> Self {
        // Load the texture
        let texture = pipec::texturec(Texture::default()
            .enable_mipmaps()
            .set_format(TextureFormat::RGBA8R)
            .apply_texture_load_options(opt)
            .cache_load(diffuse_path, asset_manager));
        self.diffuse_tex = texture;
        self
    }
    // Load the normal texture
    pub fn load_normal(mut self, normal_path: &str, opt: Option<TextureLoadOptions>, asset_manager: &mut AssetManager) -> Self {
        // Load the texture
        let texture = pipec::texturec(Texture::default()
            .enable_mipmaps()
            .set_format(TextureFormat::RGBA8R)
            .apply_texture_load_options(opt)
            .cache_load(normal_path, asset_manager));
        self.normal_tex = texture;
        self
    }
    // Set the main shader
    pub fn set_shader(mut self, shader: ShaderGPUObject) -> Self {
        self.shader = shader;
        self
    }
    // Toggle the double sided flag for this material
    pub fn set_double_sided(mut self, double_sided: bool) -> Self {
        match double_sided {
            true => self.flags.insert(MaterialFlags::DOUBLE_SIDED),
            false => self.flags.remove(MaterialFlags::DOUBLE_SIDED),
        }
        self
    }
    // Toggle the visibility of this material
    pub fn set_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

// Each material can be instanced
impl others::Instance for Material {
    fn set_name(&mut self, string: String) {
        self.material_name = string
    }
    fn get_name(&self) -> String {
        self.material_name.clone()
    }
}
