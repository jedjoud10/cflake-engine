use crate::{
    basics::{shader::Shader, uniforms::UniformsSet},
    object::Object,
    pipeline::*,
};

// A generic material that contains a shader and a set of uniforms
pub struct Material {
    // Shader that will render the surface
    pub shader: Handle<Shader>,

    // Custom uniforms
    pub uniforms: UniformsSet,
}

// Builds a universal material from anything
pub trait MaterialBuilder
where
    Self: Sized,
{
    // Builds the material with the default shader
    fn build(self, pipeline: &mut Pipeline) -> Handle<Material> {
        let shader = pipeline.defaults().shader.clone();
        self.build_with_shader(pipeline, shader)
    }
    // Build the material using a speficic shader
    fn build_with_shader(self, pipeline: &mut Pipeline, shader: Handle<Shader>) -> Handle<Material>;
}

impl Object for Material {}

impl Default for Material {
    fn default() -> Self {
        Self {
            shader: Default::default(),
            uniforms: UniformsSet::default(),
        }
    }
}
