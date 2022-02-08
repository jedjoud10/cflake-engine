use crate::object::{Construct, ConstructionTask, Deconstruct, DeconstructionTask, ObjectID, PipelineObject};
use crate::pipeline::*;

use super::shader::Shader;
use super::texture::Texture;
use super::uniforms::{Uniforms, SetUniformsCallback};
// A material that can have multiple parameters and such
#[derive(Default)]
pub struct Material {
    pub shader: ObjectID<Shader>,
    pub double_sided: bool,
    pub callback: Option<SetUniformsCallback>,
}

impl PipelineObject for Material {
    // Reserve an ID for this material
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, pipeline.materials.gen_id()))
    }
    // Send this material to the pipeline for construction
    fn send(self, _pipeline: &Pipeline, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::Material(Construct::<Self>(self, id))
    }
    // Create a deconstruction task
    fn pull(_pipeline: &Pipeline, id: ObjectID<Self>) -> DeconstructionTask {
        DeconstructionTask::Material(Deconstruct::<Self>(id))
    }
    // Add the material to our ordered vec
    fn add(mut self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // Make sure we have a valid shader
        if !self.shader.is_some() {
            self.shader = pipeline.defaults.as_ref()?.shader;
        }

        // Add the material
        pipeline.materials.insert(id, self)?;
        Some(())
    }
    // Remove the material from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        pipeline.materials.remove(id)
    }
}

// This should help us create a material
impl Material {
    // Set the main shader
    pub fn with_shader(mut self, shader: ObjectID<Shader>) -> Self {
        self.shader = shader;
        self
    }
    // Set the uniforms callback
    pub fn with_uniforms(mut self, callback: SetUniformsCallback) -> Self {
        self.callback = Some(callback);
        self
    }
}
