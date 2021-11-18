use crate::pipeline::*;
use crate::basics::*;
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
#[derive(Clone)]
pub struct Material {
    // Rendering stuff
    pub shader: GPUObject,
    pub material_name: String,
    pub flags: MaterialFlags,
    pub default_uniforms: HashMap<String, Uniform>,
    // The default textures
    pub diffuse_tex: GPUObject,
    pub normal_tex: GPUObject,
    // Is this material even visible?
    pub visible: bool,
}

impl Default for Material {
    fn default() -> Self {
        let material: Self = Material {
            shader: GPUObject::None,
            material_name: String::new(),
            flags: MaterialFlags::empty(),
            default_uniforms: HashMap::new(),
            diffuse_tex: GPUObject::None,
            normal_tex: GPUObject::None,
            visible: true,
        };
        // Set the default shader args
        let material = material.set_uniform("uv_scale", Uniform::Vec2F32(veclib::Vector2::ONE));
        let material = material.set_uniform("tint", Uniform::Vec3F32(veclib::Vector3::ONE));
        
        material.set_uniform("normals_strength", Uniform::F32(1.0))
    }
}

impl Material {
    // Create a new material with a name
    pub fn new(material_name: &str, asset_manager: &mut AssetManager) -> Self {
        Self {
            material_name: material_name.to_string(),
            diffuse_tex: Texture::object_load_o("white", &mut asset_manager.object_cacher).id,
            normal_tex: Texture::object_load_o("default_normals", &mut asset_manager.object_cacher).id,
            ..Self::default()
        }
    }
    // Load the diffuse texture
    pub fn load_diffuse(mut self, diffuse_path: &str, opt: Option<TextureLoadOptions>, asset_manager: &mut AssetManager) -> Self {
        // Load the texture
        let texture = Texture::default()
            .enable_mipmaps()
            .set_format(TextureFormat::RGBA8R)
            .apply_texture_load_options(opt)
            .cache_load(diffuse_path, asset_manager).id;
        self.diffuse_tex = texture;
        self
    }
    // Load the normal texture
    pub fn load_normal(mut self, normal_path: &str, opt: Option<TextureLoadOptions>, asset_manager: &mut AssetManager) -> Self {
        // Load the texture
        let texture = Texture::default()
            .enable_mipmaps()
            .set_format(TextureFormat::RGBA8R)
            .apply_texture_load_options(opt)
            .cache_load(normal_path, asset_manager).id;
        self.normal_tex = texture;
        self
    }
    // Set the main shader
    pub fn set_shader(mut self, shader: Rc<Shader>) -> Self {
        self.shader = Some(shader);
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
    // Set a default uniform
    pub fn set_uniform(mut self, uniform_name: &str, uniform: Uniform) -> Self {
        self.default_uniforms.insert(uniform_name.to_string(), uniform);
        self
    }
    // Set a default uniform but also it's inxed
    pub fn set_uniform_i(self, uniform_name: &str, uniform: Uniform) -> (Self, usize) {
        let i = self.default_uniforms.len();
        (self.set_uniform(uniform_name, uniform), i)
    }
    // Update a default uniform
    pub fn update_uniform(&mut self, uniform_name: &str, new_uniform: Uniform) {
        self.default_uniforms.insert(uniform_name.to_string(), new_uniform);
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
