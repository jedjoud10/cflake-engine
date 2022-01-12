use crate::{ShaderSource, object::{PipelineObject, ObjectID, PipelineTask, ObjectBuildingTask}, Buildable};

// A compute shader that can run parallel calculations on the GPU
pub struct ComputeShader {
    // The OpenGL program linked to this compute shader
    pub(crate) program: u32,
    // We only have one shader source since we are a compute shader
    pub(crate) source: ShaderSource
}
impl PipelineObject for ComputeShader {}

impl Buildable for ComputeShader {
    fn construct_task(self, pipeline: &crate::Pipeline) -> PipelineTask {
        // Create the ID
        let id = pipeline.compute_shaders.get_next_idx_increment();
        let id = ObjectID::new(id);
        PipelineTask::CreateComputeShader(ObjectBuildingTask::<Self>(self, id))
    }
}