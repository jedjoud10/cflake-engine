use super::Texture2D;
use bitflags::bitflags;
use others::CacheManager;
use resources::ResourceManager;

bitflags! {
    pub struct MaterialFlags: u8 {
        const DOUBLE_SIDED = 0b00000001;
    }
}

// A material that can have multiple parameters and such
#[derive(Debug, Clone)]
pub struct Material {
    // Rendering stuff
    pub shader_name: String,
    pub material_name: String,
    pub texture_cache_ids: Vec<u16>,
    pub uniform_setter: ShaderUniformSetter,
    pub flags: MaterialFlags,
}

impl Default for Material {
    fn default() -> Self {
        let mut material: Self = Material {
            shader_name: String::new(),
            material_name: String::new(),
            texture_cache_ids: Vec::new(),
            uniform_setter: ShaderUniformSetter::default(),
            flags: MaterialFlags::empty(),
        };
        // Set the default shader args
        let material = material.set_uniform("uv_scale", ShaderArg::V2F32(veclib::Vector2::ONE));
        let material = material.set_uniform("tint", ShaderArg::V3F32(veclib::Vector3::ONE));
        let material = material.set_uniform("normals_strength", ShaderArg::F32(1.0));
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
    // Load textures from their texture struct
    pub fn load_textures(mut self, texture_ids: &Vec<u16>, texture_cacher: &CacheManager<Texture2D>) -> Self {
        // Set the textures as the renderer's textures
        for (&texture_id) in texture_ids.iter() {
            // Since these are loadable textures, we already know they got cached beforehand
            self.texture_cache_ids.push(texture_id);
        }
        // Load the default textures
        return self.load_default_textures(texture_cacher);
    }
    // Load the default textures
    pub fn load_default_textures(mut self, texture_cacher: &CacheManager<Texture2D>) -> Self {
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
        texture_cacher: &mut CacheManager<Texture2D>,
        resource_manager: &mut ResourceManager,
    ) -> Result<Self, errors::ResourceError> {
        // Load the textures
        for (_i, &texture_path) in texture_paths.iter().enumerate() {
            let _resource = resource_manager.load_packed_resource(texture_path)?;
            let _texture = Texture2D::new()
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
    // Toggle the double sided flag for this material
    pub fn set_double_sided(mut self, double_sided: bool) -> Self {
        match double_sided {
            true => self.flags.insert(MaterialFlags::DOUBLE_SIDED),
            false => self.flags.remove(MaterialFlags::DOUBLE_SIDED),
        }
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

// Used to manually set some uniforms for the shaders
#[derive(Default, Debug, Clone)]
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
#[derive(Debug, Clone)]
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
