use crate::{pipeline::*, basics::uniforms::SetUniformsCallback};
// A material that can have multiple parameters and such
pub struct Material {
    // Main settings
    pub shader: ShaderKey,
    pub(crate) uniforms: SetUniformsCallback,

    // Actual parameters used for rendering
    pub diffuse_map: TextureKey,
    pub normal_map: TextureKey,
    pub emissive_map: TextureKey,
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
            diffuse_map: Default::default(),
            normal_map: Default::default(),
            emissive_map: Default::default(),
            tint: veclib::Vector3::ONE,
            normal_map_strength: 1.0,
            emissive_map_strength: 1.0,
            uv_scale: veclib::Vector2::ONE,
        }
    }
}