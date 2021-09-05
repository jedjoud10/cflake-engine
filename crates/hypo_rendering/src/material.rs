use hypo_others::CacheManager;
use hypo_resources::ResourceManager;
use super::{Texture};

// A material that can have multiple parameters and such
#[derive(Default)]
pub struct Material {
    // Rendering stuff
    pub shader_name: String,    
    pub texture_cache_ids: Vec<u16>,
    pub uniform_setter: ShaderUniformSetter,
    pub uv_scale: veclib::Vector2<f32>,    
}

impl Material {    
    // Set the uv scale
    pub fn set_uv_scale(mut self, new_scale: veclib::Vector2<f32>) -> Self {
        self.uv_scale = new_scale;
        return self;
    }
     // Load textures from their texture struct
     pub fn load_textures(mut self, texture_ids: Vec<u16>, texture_cacher: &CacheManager<Texture>) -> Self {
        // Set the textures as the renderer's textures
        for (&texture_id) in texture_ids.iter() {
            // Since these are loadable textures, we already know they got cached beforehand
            self.texture_cache_ids.push(texture_id);
        }
        // Load the default textures
        return self.load_default_textures(texture_cacher);
    }
    // Load the default textures
    pub fn load_default_textures(mut self, texture_cacher: &CacheManager<Texture>) -> Self {
        // For the rest of the textures that weren't explicitly given a texture path, load the default ones
        // Diffuse, Normals, Roughness, Metallic, AO
        for _i in (self.texture_cache_ids.len())..5 {
            self.texture_cache_ids.push(texture_cacher.get_object_id("defaults\\textures\\white.png").unwrap());
        }
        return self;
    }
    // Set a specific uniform, wrapper around ShaderUniformSetter
    pub fn set_uniform(mut self, uniform_name: &str, value: ShaderArg) -> Self {
        self.uniform_setter.set_uniform(uniform_name, value);
        return self;
    }
    // Load textures from their resource paths
    pub fn resource_load_textures(
        mut self,
        texture_paths: Vec<&str>,
        texture_cacher: &mut CacheManager<Texture>,
        resource_manager: &mut ResourceManager,
    ) -> Result<Self, hypo_errors::ResourceError> {
        // Load the textures
        for (_i, &texture_path) in texture_paths.iter().enumerate() {
            let _resource = resource_manager.load_packed_resource(texture_path)?;
            let _texture = Texture::new()
                .set_mutable(true)
                .enable_mipmaps()
                .set_idf(gl::RGBA, gl::RGBA, gl::UNSIGNED_BYTE)
                .load_texture(texture_path, resource_manager, texture_cacher)
                .unwrap();
            self.texture_cache_ids.push(texture_cacher.get_object_id(texture_path).unwrap());
        }
        // Load the default textures
        return Ok(self.load_default_textures(texture_cacher));
    }
    // Set the main shader
    pub fn set_shader(mut self, shader_name: &str) -> Self {
        self.shader_name = shader_name.to_string();
        return self;
    }
}

// Used to manually set some uniforms for the shaders
#[derive(Default)]
pub struct ShaderUniformSetter {
    // The arguments that are going to be written to
    pub uniforms: Vec<(String, ShaderArg)>,
}

impl ShaderUniformSetter {
    // Set a specific uniform to a specific value
    pub fn set_uniform(&mut self, uniform_name: &str, value: ShaderArg) {
        self.uniforms.push((uniform_name.to_string(), value));
    }
}

// The type of shader argument
pub enum ShaderArg {
    F32(f32),
    I32(i32),
    V2F32(veclib::Vector2<f32>),
    V3F32(veclib::Vector3<f32>),
    V4F32(veclib::Vector4<f32>),
    V2I32(veclib::Vector2<i32>),
    V3I32(veclib::Vector3<i32>),
    V4I32(veclib::Vector4<i32>),
    MAT44(veclib::Matrix4x4<f32>),
}
