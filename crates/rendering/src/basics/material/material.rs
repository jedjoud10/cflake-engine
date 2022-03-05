use crate::{pipeline::*, basics::{uniforms::SetUniformsCallback, texture::Texture, shader::Shader}};

// Material textures
#[derive(Default)]
pub struct MaterialTextures {
    pub diffuse_map: Handle<Texture>,
    pub normal_map: Handle<Texture>,
    pub emissive_map: Handle<Texture>,
}

// A material that can have multiple parameters and such
pub struct Material {
    // Main settings
    pub shader: Handle<Shader>,
    pub(crate) uniforms: SetUniformsCallback,

    // Actual parameters used for rendering
    pub textures: MaterialTextures,
    pub tint: veclib::Vector3<f32>,
    pub normal_map_strength: f32,
    pub emissive_map_strength: f32,
    pub uv_scale: veclib::Vector2<f32>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            shader: Default::default(),
            uniforms: Default::default(),
            textures: Default::default(),
            tint: veclib::Vector3::ONE,
            normal_map_strength: 1.0,
            emissive_map_strength: 1.0,
            uv_scale: veclib::Vector2::ONE,
        }
    }
}