use crate::{
    basics::{shader::Shader, uniforms::UniformsSet},
    object::ObjectSealed,
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
// TODO: Remove this and use the default pipeline shader storage instead.
// This should allow for simpler logic and shader caching 
pub trait MaterialBuilder
where
    Self: Sized,
{
    // Fetch the shader handle that corresponds to this material
    fn shader(pipeline: &mut Pipeline) -> Handle<Shader>;

    // Build the material by fetching the shader handle internally
    fn build(self, pipeline: &mut Pipeline) -> Handle<Material> {
        let shader = Self::shader(pipeline);
        self.build_with(pipeline, shader)
    }

    // Build the material using the corresponding shader
    fn build_with(self, pipeline: &mut Pipeline, shader: Handle<Shader>) -> Handle<Material>;
}

impl ObjectSealed for Material {}

impl Default for Material {
    fn default() -> Self {
        Self {
            shader: Default::default(),
            uniforms: UniformsSet::default(),
        }
    }
}
