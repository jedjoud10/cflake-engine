use std::collections::HashMap;

use crate::{Texture, TextureFilter};

use bitflags::bitflags;
use others::CacheManager;
use resources::ResourceManager;

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
    pub diffuse_tex_id: Option<usize>,
    pub normal_tex_id: Option<usize>,    
}

// Some default uniforms that we will set
#[derive(Clone)]
pub enum DefaultUniform {
    // Singles
    F32(f32),
    I32(i32),
    // Vectors
    Vec2F32(veclib::Vector2<f32>),
    Vec3F32(veclib::Vector3<f32>),
    Vec4F32(veclib::Vector4<f32>),
    Vec2I32(veclib::Vector2<i32>),
    Vec3I32(veclib::Vector3<i32>),
    Vec4I32(veclib::Vector4<i32>),
    Mat44F32(veclib::Matrix4x4<f32>),
    // Others
    Texture2D(usize, u32),
    Texture3D(usize, u32),
}


impl Default for Material {
    fn default() -> Self {
        let material: Self = Material {
            shader_name: String::new(),
            material_name: String::new(),
            flags: MaterialFlags::empty(),
            default_uniforms: Vec::new(),
            diffuse_tex_id: None,
            normal_tex_id: None,
        };
        // Set the default shader args
        let material = material.set_uniform("uv_scale", DefaultUniform::Vec2F32(veclib::Vector2::ONE));
        let material = material.set_uniform("tint", DefaultUniform::Vec3F32(veclib::Vector3::ONE));
        let material = material.set_uniform("normals_strength", DefaultUniform::F32(1.0));
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
    pub fn load_diffuse(mut self, diffuse_path: &str, texture_cacher: &mut CacheManager<Texture>, resource_manager: &mut ResourceManager) -> Self {
        // Load the texture
        let (_, id) = Texture::new()
            .set_mutable(true)
            .enable_mipmaps()
            .set_idf(gl::RGBA, gl::RGBA, gl::UNSIGNED_BYTE)
            .set_filter(TextureFilter::Nearest)
            .load_texture(diffuse_path, resource_manager, texture_cacher)
            .unwrap();
        self.diffuse_tex_id = Some(id);
        return self;
    }
    // Load the normal texture
    pub fn load_normal(mut self, normal_path: &str, texture_cacher: &mut CacheManager<Texture>, resource_manager: &mut ResourceManager) -> Self {
        // Load the texture
        let (_, id) = Texture::new()
            .set_mutable(true)
            .enable_mipmaps()
            .set_idf(gl::RGBA, gl::RGBA, gl::UNSIGNED_BYTE)
            .load_texture(normal_path, resource_manager, texture_cacher)
            .unwrap();
        self.normal_tex_id = Some(id);
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
    // Set a default uniform
    pub fn set_uniform(mut self, uniform_name: &str, uniform: DefaultUniform) -> Self {
        self.default_uniforms.push((uniform_name.to_string(), uniform));
        return self;
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
