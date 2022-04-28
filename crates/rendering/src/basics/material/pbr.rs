use crate::{
    basics::{
        material::{Material, MaterialType},
        shader::{Shader, ShaderInitSettings},
        texture::Texture2D,
    },
    pipeline::{Handle, Pipeline},
};

// A physically based material that shall be stored in the pipeline
pub struct PbrMaterial {
    // Textures
    pub diffuse: Handle<Texture2D>,
    pub normal: Handle<Texture2D>,
    pub emissive: Handle<Texture2D>,
    
    // R: AO
    // G: Roughness
    // B: Metallic
    pub mask: Handle<Texture2D>,

    // Parameters
    pub bumpiness: f32,
    pub emissivity: f32,
    pub tint: vek::Rgb<f32>,
    pub scale: vek::Vec2<f32>,
    pub ao: f32,
    pub roughness: f32,
    pub metallic: f32,
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self {
            diffuse: Default::default(),
            normal: Default::default(),
            emissive: Default::default(),
            mask: Default::default(),
            bumpiness: 1.0,
            emissivity: 0.0,
            tint: vek::Rgb::one(),
            scale: vek::Vec2::one(),
            ao: 1.0,
            roughness: 1.0,
            metallic: 1.0,
        }
    }
}

impl MaterialType for PbrMaterial {
    // Get the defauflt PBR shader
    fn shader(&self, pipeline: &Pipeline) -> Handle<Shader> {
        pipeline.defaults().pbr_shader.clone()
    }

    // Set the PBR material uniforms
    fn set(&self, pipeline: &Pipeline, uniforms: &mut crate::basics::uniforms::Uniforms) {
        // Fallback to the default textures if needed
        let diffuse = self.diffuse.fallback_to(&pipeline.defaults().white);
        let normal = self.normal.fallback_to(&pipeline.defaults().normal_map);
        let emissive = self.emissive.fallback_to(&pipeline.defaults().normal_map);
        let mask = self.mask.fallback_to(&pipeline.defaults().mask);

        // Set the main texture maps
        uniforms.set_texture2d("diffuse_m", diffuse);
        uniforms.set_texture2d("normal_m", normal);
        uniforms.set_texture2d("emissive_m", emissive);
        uniforms.set_texture2d("mask_m", mask);

        // Set the PBR parameters
        uniforms.set_vec3f32("tint", self.tint.into());
        uniforms.set_f32("bumpiness", self.bumpiness);
        uniforms.set_f32("emissivity", self.emissivity);
        uniforms.set_f32("roughness", self.roughness);
        uniforms.set_f32("metallic", self.metallic);
        uniforms.set_f32("ao_strength", self.ao);
        uniforms.set_vec2f32("uv_scale", self.scale);
    }
}
