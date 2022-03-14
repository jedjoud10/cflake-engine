use crate::{
    basics::{
        material::{Material, MaterialBuilder},
        shader::Shader,
        texture::Texture2D,
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
        let textures = self.textures;
        let params = self.params;

        Material {
            shader,
            uniforms: UniformsSet::new(move |uniforms| {
                // Set the textures first
                uniforms.set_texture2d("diffuse_m", &textures.diffuse);
                uniforms.set_texture2d("normal_m", &textures.normal);
                uniforms.set_texture2d("emissive_m", &textures.emissive);
                // Then the parameters
                uniforms.set_vec3f32("tint", params.tint);
                uniforms.set_f32("bumpiness", params.bumpiness);
                uniforms.set_f32("emissivity", params.emissivity);
                uniforms.set_vec2f32("uv_scale", params.uv_scale);
            }),
        }
    }
}
