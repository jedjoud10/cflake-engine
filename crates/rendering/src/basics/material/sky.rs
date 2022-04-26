use crate::{
    basics::{
        material::{Material, MaterialBuilder},
        shader::Shader,
        texture::{Texture2D, CubeMap},
        uniforms::UniformsSet,
    },
    pipeline::Handle,
};

// A sky material that we can use to uh, render a sky
pub struct SkyMaterialBuilder {
    cubemap: Handle<CubeMap>,
}

impl Default for SkyMaterialBuilder {
    fn default() -> Self {
        Self {
            cubemap: Default::default()
        }
    }
}

impl SkyMaterialBuilder {
    // Set the cubemap that we will sample throughout the fragment shader
    pub fn cubemap(mut self, cubemap: Handle<CubeMap>) -> Self {
        self.cubemap = cubemap;
        self
    }
}

impl MaterialBuilder for SkyMaterialBuilder {
    fn shader(pipeline: &mut crate::pipeline::Pipeline) -> Handle<Shader> {
        todo!()
    }
    fn build_with(self, pipeline: &mut crate::pipeline::Pipeline, shader: Handle<Shader>) -> Handle<Material> {
        todo!()
    }
}
