use std::collections::HashSet;

use crate::{basics::shader::{ShaderSource, ShaderSettings}, object::{PipelineObject, ObjectID, ConstructionTask, Construct}, pipeline::Pipeline, utils::RenderingError};

// A compute shader that can run parallel calculations on the GPU
pub struct ComputeShader {
    // The OpenGL program linked to this compute shader
    pub(crate) program: u32,
    // We only have one shader source since we are a compute shader
    pub(crate) source: ShaderSource,
}

impl PipelineObject for ComputeShader {
    // Reserve an ID for this compute shader
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, ObjectID::new(pipeline.compute_shaders.get_next_id_increment())))
    }
    // Send this compute shader to the pipeline for construction
    fn send(self, pipeline: &Pipeline, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::ComputeShader(Construct::<Self>(self, id))
    }
    // Add the compute shader to our ordered vec
    fn add(mut self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // Add the compute shader
        pipeline.compute_shaders.insert(id.get()?, self);
        Some(())
    }
    // Remove the compute shader from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        pipeline.compute_shaders.remove(id.get()?)
    }
}

impl ComputeShader {
    // Creates a compute shader from it's corresponding shader settings
    // TODO: Create a main Shader compilation error
    pub fn new(mut settings: ShaderSettings) -> Result<Self, RenderingError> {
        let mut included_paths: HashSet<String> = HashSet::new();
        // Loop through the shader sources and edit them
        let mut sources = std::mem::take(&mut settings.sources);
        // Since this is a compute shader, we only have one source
        // We won't actually generate any subshaders here, so we don't need anything related to the pipeline
        // Include the includables until they cannot be included
        let (_, mut source_data) = sources.drain().collect::<Vec<_>>().remove(0);
        while crate::basics::shader::load_includes(&settings, &mut source_data.text, &mut included_paths)? {
            // We are still including paths
        }
        // Add this shader source to be generated as a subshader
        Ok(Self { program: 0, source: source_data })
    }
}
