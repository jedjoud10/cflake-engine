use crate::{pipeline::{TextureKey, ShaderKey}, basics::uniforms::SetUniformsCallback};

use super::Material;

// Material builder
pub struct MaterialBuilder {
    inner: Material
}

impl MaterialBuilder {
    // Set the main shader
    pub fn with_shader(mut self, shader: ShaderKey) -> Self {
        self.inner.shader = shader;
        self
    }
    // Set the uniforms callback
    pub fn with_uniforms(mut self, callback: SetUniformsCallback) -> Self {
        self.inner.uniforms = callback;
        self
    }
    // With some parameters
    // Maps
    pub fn with_diffuse(mut self, diffuse_map: TextureKey) -> Self {
        self.inner.diffuse_map = diffuse_map;
        self
    }
    pub fn with_normal(mut self, normal_map: TextureKey) -> Self {
        self.inner.normal_map = normal_map;
        self
    }
    pub fn with_emissive(mut self, emissive_map: TextureKey) -> Self {
        self.inner.emissive_map = emissive_map;
        self
    }
    // Values
    pub fn with_normal_strength(mut self, strength: f32) -> Self {
        self.inner.normal_map_strength = strength;
        self
    }
    pub fn with_emissive_strenhgth(mut self, strength: f32) -> Self {
        self.inner.emissive_map_strength = strength;
        self
    }
    pub fn with_tint(mut self, tint: veclib::Vector3<f32>) -> Self {
        self.inner.tint = tint;
        self
    }
    pub fn with_uv_scale(mut self, scale: veclib::Vector2<f32>) -> Self {
        self.inner.uv_scale = scale;
        self
    }
}