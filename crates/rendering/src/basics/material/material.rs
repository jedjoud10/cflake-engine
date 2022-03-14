use crate::{
    basics::{
        shader::Shader,
        uniforms::{Uniforms, UniformsSet},
    },
    object::PipelineElement,
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
    // Build
    fn build(self, pipeline: &Pipeline) -> Material {
        // Use the default shader
        let shader = pipeline.defaults().shader.clone();
        self.build_with_shader(pipeline, shader)
    }
    // Build the material using a speficic shader
    fn build_with_shader(self, pipeline: &Pipeline, shader: Handle<Shader>) -> Material;
}

impl PipelineElement for Material {
    fn add(self, pipeline: &mut Pipeline) -> Handle<Self> {
        pipeline.materials.insert(self)
    }

    fn find<'a>(pipeline: &'a Pipeline, handle: &Handle<Self>) -> Option<&'a Self> {
        pipeline.materials.get(handle)
    }

    fn find_mut<'a>(pipeline: &'a mut Pipeline, handle: &Handle<Self>) -> Option<&'a mut Self> {
        pipeline.materials.get_mut(handle)
    }

    fn disposed(self) {}
}

impl Default for Material {
    fn default() -> Self {
        Self {
            shader: Default::default(),
            uniforms: UniformsSet::default(),
        }
    }
}
