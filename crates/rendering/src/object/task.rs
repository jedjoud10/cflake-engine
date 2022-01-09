use crate::{Texture, Material, Buildable, Shader, Model, compute::{ComputeShader, ComputeShaderExecutionSettings}, Renderer};
use super::{ObjectID, PipelineObject};

// A task to create an object
pub(crate) struct ObjectBuildingTask<T: PipelineObject + Buildable>(pub T, pub ObjectID<T>);
// A pipeline task that will be sent to the render thread
pub enum PipelineTask {
    // Creation tasks
    CreateTexture(ObjectBuildingTask<Texture>),
    CreateMaterial(ObjectBuildingTask<Material>),
    CreateShader(ObjectBuildingTask<Shader>),
    CreateComputeShader(ObjectBuildingTask<ComputeShader>),
    CreateModel(ObjectBuildingTask<Model>),
    CreateRenderer(ObjectBuildingTask<Renderer>),

    RunComputeShader(ObjectID<ComputeShader>, ComputeShaderExecutionSettings),

    // Specific pipeline tasks
    Quit,
}

// The status for a specific PipelineTask
pub enum PipelineTaskStatus {
    Pending,
    Running,
    Finished,
}