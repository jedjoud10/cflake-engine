use super::{ObjectID, PipelineObject};
use crate::{
    compute::{ComputeShader, ComputeShaderExecutionSettings},
    Buildable, Material, Model, Renderer, Shader, Texture,
};

// A task to create an object
pub struct ObjectBuildingTask<T: PipelineObject + Buildable>(pub T, pub ObjectID<T>);
// A pipeline task that will be sent to the render thread
pub enum PipelineTask {
    // Creation tasks
    CreateTexture(ObjectBuildingTask<Texture>),
    CreateMaterial(ObjectBuildingTask<Material>),
    CreateShader(ObjectBuildingTask<Shader>),
    CreateComputeShader(ObjectBuildingTask<ComputeShader>),
    CreateModel(ObjectBuildingTask<Model>),
    CreateRenderer(ObjectBuildingTask<Renderer>),

    // Update tasks
    RunComputeShader(ObjectID<ComputeShader>, ComputeShaderExecutionSettings),
    UpdateRendererMatrix(ObjectID<Renderer>, veclib::Matrix4x4<f32>),

    // Specific pipeline tasks
    Quit,
}

// The status for a specific PipelineTask
pub enum PipelineTaskStatus {
    Pending,
    Running,
    Finished,
}
