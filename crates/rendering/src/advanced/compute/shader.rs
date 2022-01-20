use std::collections::HashSet;

use crate::{
    basics::{shader::{ShaderSource, ShaderSettings, ShaderFlags}, Buildable},
    object::{ObjectBuildingTask, ObjectID, PipelineObject, PipelineTask}, utils::RenderingError,
};

// A compute shader that can run parallel calculations on the GPU
pub struct ComputeShader {
    // The OpenGL program linked to this compute shader
    pub(crate) program: u32,
    // We only have one shader source since we are a compute shader
    pub(crate) source: ShaderSource,
    // Some shader flags
    pub(crate) flags: ShaderFlags,
}
impl PipelineObject for ComputeShader {}

impl Buildable for ComputeShader {
    fn construct_task(self, pipeline: &crate::pipeline::Pipeline) -> (PipelineTask, ObjectID<Self>) {
        // Create the ID
        let id = pipeline.compute_shaders.get_next_id_increment();
        let id = ObjectID::new(id);
        (PipelineTask::CreateComputeShader(ObjectBuildingTask::<Self>(self, id)), id)
    }
}

impl ComputeShader {
    // Creates a compute shader from it's corresponding shader settings
    pub fn new(mut settings: ShaderSettings) -> Result<Self, RenderingError> {
        let mut included_paths: HashSet<String> = HashSet::new();
        // Loop through the shader sources and edit them
        let mut sources = std::mem::take(&mut settings.sources);
        // Since this is a compute shader, we only have one source
        // We won't actually generate any subshaders here, so we don't need anything related to the pipeline
        // Include the includables until they cannot be included
        let (_, mut source_data) = sources.drain().collect::<Vec<_>>().remove(0);
        let mut flags = ShaderFlags::NONE;
        while crate::basics::shader::load_includes(&mut flags, &settings, &mut source_data.text, &mut included_paths)? {
            // We are still including paths
        }
        // Add this shader source to be generated as a subshader
        Ok(Self {
            program: 0,
            source: source_data,
            flags,
        })
    }
}