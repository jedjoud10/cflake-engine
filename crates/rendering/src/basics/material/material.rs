use super::MaterialTextures;
use crate::{
    basics::{shader::Shader, texture::Texture},
    object::PipelineCollectionElement,
    pipeline::*,
};

// A material that can have multiple parameters and such
pub struct Material {
    // Main settings
    pub shader: Handle<Shader>,

    // Actual parameters used for rendering
    pub textures: MaterialTextures,
    pub tint: veclib::Vector3<f32>,
    pub normal_map_strength: f32,
    pub emissive_map_strength: f32,
    pub uv_scale: veclib::Vector2<f32>,
}

impl PipelineCollectionElement for Material {
    fn added(&mut self, handle: &Handle<Self>) {}

    fn disposed(self) {}
}

impl Default for Material {
    fn default() -> Self {
        Self {
            shader: Default::default(),
            textures: Default::default(),
            tint: veclib::Vector3::ONE,
            normal_map_strength: 1.0,
            emissive_map_strength: 1.0,
            uv_scale: veclib::Vector2::ONE,
        }
    }
}
