use crate::{Shader, Texture, TextureFilter, TextureLoadOptions, TextureWrapping, Uniform};
use assets::{Asset, AssetManager, Object};
use bitflags::bitflags;
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
    pub shader: Option<Rc<Shader>>,
    pub material_name: String,
    pub flags: MaterialFlags,
    pub default_uniforms: HashMap<String, Uniform>,
    // The default texture ID
    pub diffuse_tex: Option<Rc<Texture>>,
    pub normal_tex: Option<Rc<Texture>>,
    // Is this material even visible?
    pub visible: bool,
}

impl Default for Material {
    fn default() -> Self {
        let material: Self = Material {
            shader: None,
            material_name: String::new(),
            flags: MaterialFlags::empty(),
            default_uniforms: HashMap::new(),
            diffuse_tex: None,
            normal_tex: None,
            visible: true,
        };
        // Set the default shader args
        let material = material.set_uniform("uv_scale", Uniform::Vec2F32(veclib::Vector2::ONE));
        let material = material.set_uniform("tint", Uniform::Vec3F32(veclib::Vector3::ONE));
        let material = material.set_uniform("normals_strength", Uniform::F32(1.0));
        return material;
    }
}

impl Material {
    // Create a new material with a name
    pub fn new(material_name: &str, asset_manager: &mut AssetManager) -> Self {
        Self {
            material_name: material_name.to_string(),
            diffuse_tex: Some(Texture::object_load_o("white", &mut asset_manager.object_cacher)),
            normal_tex: Some(Texture::object_load_o("default_normals", &mut asset_manager.object_cacher)),
            ..Self::default()
        }
    }
    // Load the diffuse texture
    pub fn load_diffuse(mut self, diffuse_path: &str, opt: Option<TextureLoadOptions>, asset_manager: &mut AssetManager) -> Self {
        // Load the texture
        let rc_texture = Texture::new()
            .enable_mipmaps()
            .set_idf(gl::RGBA, gl::RGBA, gl::UNSIGNED_BYTE)
            .apply_texture_load_options(opt)
            .cache_load(diffuse_path, asset_manager);
        self.diffuse_tex = Some(rc_texture);
        return self;
    }
    // Load the normal texture
    pub fn load_normal(mut self, normal_path: &str, opt: Option<TextureLoadOptions>, asset_manager: &mut AssetManager) -> Self {
        // Load the texture
        let rc_texture = Texture::new()
            .enable_mipmaps()
            .set_idf(gl::RGBA, gl::RGBA, gl::UNSIGNED_BYTE)
            .apply_texture_load_options(opt)
            .cache_load(normal_path, asset_manager);
        self.normal_tex = Some(rc_texture);
        return self;
    }
    // Set the main shader
    pub fn set_shader(mut self, shader: Rc<Shader>) -> Self {
        self.shader = Some(shader);
        return self;
    }
    // Toggle the double sided flag for this material
    pub fn set_double_sided(mut self, double_sided: bool) -> Self {
        match double_sided {
            true => self.flags.insert(MaterialFlags::DOUBLE_SIDED),
            false => self.flags.remove(MaterialFlags::DOUBLE_SIDED),
        }
        return self;
    }
    // Toggle the visibility of this material
    pub fn set_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        return self;
    }
    // Set a default uniform
    pub fn set_uniform(mut self, uniform_name: &str, uniform: Uniform) -> Self {
        self.default_uniforms.insert(uniform_name.to_string(), uniform);
        return self;
    }
    // Set a default uniform but also it's inxed
    pub fn set_uniform_i(mut self, uniform_name: &str, uniform: Uniform) -> (Self, usize) {
        let i = self.default_uniforms.len();
        return (self.set_uniform(uniform_name, uniform), i);
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
