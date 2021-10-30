use std::{collections::HashMap, rc::Rc};
use crate::{DefaultUniform, Texture, TextureFilter, TextureLoadOptions, TextureWrapping};
use assets::{Asset, AssetManager};
use bitflags::bitflags;

bitflags! {
    pub struct MaterialFlags: u8 {
        const DOUBLE_SIDED = 0b00000001;
    }
}

// A material that can have multiple parameters and such
#[derive(Clone)]
pub struct Material {
    // Rendering stuff
    pub shader_name: String,
    pub material_name: String,
    pub flags: MaterialFlags,
    pub default_uniforms: Vec<(String, DefaultUniform)>,
    // The default texture ID
    pub diffuse_tex: Option<Rc<Texture>>,
    pub normal_tex: Option<Rc<Texture>>,
    // Is this material even visible?
    pub visible: bool,
}

impl Default for Material {
    fn default() -> Self {
        let material: Self = Material {
            shader_name: String::new(),
            material_name: String::new(),
            flags: MaterialFlags::empty(),
            default_uniforms: Vec::new(),
            diffuse_tex: None,
            normal_tex: None,
            visible: true,
        };
        // Set the default shader args
        let material = material.set_uniform("uv_scale", DefaultUniform::Vec2F32(veclib::Vector2::ONE)).0;
        let material = material.set_uniform("tint", DefaultUniform::Vec3F32(veclib::Vector3::ONE)).0;
        let material = material.set_uniform("normals_strength", DefaultUniform::F32(1.0)).0;
        return material;
    }
}

impl Material {
    // Create a new material with a name
    pub fn new(material_name: &str) -> Self {
        Self {
            material_name: material_name.to_string(),
            ..Self::default()
        }
    }
    // Load the diffuse texture
    pub fn load_diffuse(mut self, diffuse_path: &str, opt: Option<TextureLoadOptions>, asset_manager: &AssetManager) -> Self {
        // Load the texture
        let texture = Texture::new()
            .enable_mipmaps()
            .set_idf(gl::RGBA, gl::RGBA, gl::UNSIGNED_BYTE)
            .apply_texture_load_options(opt)
            .load_t(asset_manager.load(diffuse_path).unwrap());
        return self;
    }
    // Load the normal texture
    pub fn load_normal(mut self, normal_path: &str, opt: Option<TextureLoadOptions>, asset_manager: &AssetManager) -> Self {
        // Load the texture
        let texture = Texture::new()
            .enable_mipmaps()
            .set_idf(gl::RGBA, gl::RGBA, gl::UNSIGNED_BYTE)
            .apply_texture_load_options(opt)
            .load_t(asset_manager.load(normal_path).unwrap());
        self.diffuse_tex = Some(Rc::new)
        return self;
    }
    // Load textures from their texture struct
    pub fn load_textures(mut self, texture_ids: &Vec<Option<usize>>, texture_cacher: &CacheManager<Texture>) -> Self {
        self.diffuse_tex_id = texture_ids[0];
        self.normal_tex_id = texture_ids[1];
        // Load the default textures
        return self.load_default_textures(texture_cacher);
    }
    // Load the default textures
    pub fn load_default_textures(mut self, texture_cacher: &CacheManager<Texture>) -> Self {
        // For the rest of the textures that weren't explicitly given a texture path, load the default ones
        // Diffuse, Normals
        if self.diffuse_tex_id.is_none() {
            self.diffuse_tex_id = Some(texture_cacher.get_object_id("white").unwrap());
        }
        if self.normal_tex_id.is_none() {
            self.normal_tex_id = Some(texture_cacher.get_object_id("default_normals").unwrap());
        }
        return self;
    }
    // Load textures from their resource paths
    pub fn resource_load_textures(
        mut self,
        texture_paths: Vec<Option<&str>>,
        opt: Option<TextureLoadOptions>,
        texture_cacher: &mut CacheManager<Texture>,
        resource_manager: &mut ResourceManager,
    ) -> Result<Self, errors::ResourceError> {
        // Load the textures
        for (i, &texture_path) in texture_paths.iter().enumerate() {
            match texture_path {
                Some(texture_path) => {
                    let _resource = resource_manager.load_packed_resource(texture_path)?;
                    let (_, texture_id) = Texture::new()
                        .enable_mipmaps()
                        .set_idf(gl::RGBA, gl::RGBA, gl::UNSIGNED_BYTE)
                        .apply_texture_load_options(opt)
                        .load_texture(texture_path, resource_manager, texture_cacher)
                        .unwrap();
                    match i {
                        0 => {
                            self.diffuse_tex_id = Some(texture_id);
                        }
                        1 => {
                            self.normal_tex_id = Some(texture_id);
                        }
                        _ => {}
                    }
                }
                None => {}
            }
        }

        // Load the default textures
        return Ok(self.load_default_textures(texture_cacher));
    }
    // Set the main shader
    pub fn set_shader(mut self, shader_name: &str) -> Self {
        self.shader_name = shader_name.to_string();
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
    pub fn set_uniform(mut self, uniform_name: &str, uniform: DefaultUniform) -> (Self, usize) {
        let i = self.default_uniforms.len();
        self.default_uniforms.push((uniform_name.to_string(), uniform));
        return (self, i);
    }
    // Update a default uniform
    pub fn update_uniform(&mut self, uniform_index: usize, new_uniform: DefaultUniform) {
        let name = self.default_uniforms.get(uniform_index).unwrap().0.clone();
        self.default_uniforms[uniform_index] = (name, new_uniform);
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
