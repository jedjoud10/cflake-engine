use crate::{basics::texture::Texture2D, pipeline::Handle};

// PBR textures
pub struct PbrTextures {
    // Textures
    pub diffuse: Handle<Texture2D>,
    pub normal: Handle<Texture2D>,
    pub emissive: Handle<Texture2D>,
}

// PBR texture settings
pub struct PbrParams {
    // Parameters
    pub bumpiness: f32,
    pub emissivity: f32,
    pub tint: vek::Vec3<f32>,
    pub uv_scale: vek::Vec2<f32>,
}