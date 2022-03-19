use crate::{
    basics::{
        material::{Material, MaterialBuilder},
        shader::Shader,
        uniforms::UniformsSet,
    },
    pipeline::Handle,
};

use super::{PbrParams, PbrTextures};

// A physically based material builder
pub struct PbrMaterialBuilder {
    // Textures
    pub textures: PbrTextures,

    // Parameters
    pub params: PbrParams,
}

// Convert
impl MaterialBuilder for PbrMaterialBuilder {
    fn build_with_shader(self, pipeline: &crate::pipeline::Pipeline, shader: Handle<Shader>) -> Material {
        // Clone the default texture handles
        let white = pipeline.defaults().white.clone();
        let black = pipeline.defaults().black.clone();
        let normal_map = pipeline.defaults().normal_map.clone();

        let textures = self.textures;
        let params = self.params;

        Material {
            shader,
            uniforms: UniformsSet::new(move |mut uniforms| {
                // Use default textures if we need to
                let diffuse = textures.diffuse.fallback_to(&white);
                let normal = textures.normal.fallback_to(&normal_map);
                let emissive = textures.emissive.fallback_to(&black);
                uniforms.set_texture2d("diffuse_m", diffuse);
                uniforms.set_texture2d("normal_m", normal);
                uniforms.set_texture2d("emissive_m", emissive);
                // Then the parameters
                uniforms.set_vec3f32("tint", params.tint);
                uniforms.set_f32("bumpiness", params.bumpiness);
                uniforms.set_f32("emissivity", params.emissivity);
                uniforms.set_vec2f32("uv_scale", params.uv_scale);
            }),
        }
    }
}
