use super::{ObjectID, PipelineObject, TrackingTaskID};
use crate::{
    advanced::compute::{ComputeShader, ComputeShaderExecutionSettings},
    basics::{material::Material, model::Model, renderer::Renderer, shader::Shader, texture::Texture, Buildable},
    pipeline::camera::Camera,
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
    CreateModel(ObjectBuildingTask<Model>,),
    CreateRenderer(ObjectBuildingTask<Renderer>),

    // Update tasks
    RunComputeShader(ObjectID<ComputeShader>, ComputeShaderExecutionSettings),
    UpdateRendererMatrix(ObjectID<Renderer>, veclib::Matrix4x4<f32>),
    UpdateCamera(Camera),
    // Specific pipeline tasks
}

// Bruh
pub enum PipelineTaskCombination {
    Single(PipelineTask),
    SingleTracked(PipelineTask, TrackingTaskID),
    Batch(Vec<PipelineTask>),
}

impl std::fmt::Debug for PipelineTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateTexture(arg0) => f.debug_tuple("CreateTexture").finish(),
            Self::CreateMaterial(arg0) => f.debug_tuple("CreateMaterial").finish(),
            Self::CreateShader(arg0) => f.debug_tuple("CreateShader").finish(),
            Self::CreateComputeShader(arg0) => f.debug_tuple("CreateComputeShader").finish(),
            Self::CreateModel(arg0) => f.debug_tuple("CreateModel").finish(),
            Self::CreateRenderer(arg0) => f.debug_tuple("CreateRenderer").finish(),
            Self::RunComputeShader(arg0, arg1) => f.debug_tuple("RunComputeShader").finish(),
            Self::UpdateRendererMatrix(arg0, arg1) => f.debug_tuple("UpdateRendererMatrix").finish(),
            Self::UpdateCamera(arg0) => f.debug_tuple("UpdateCamera").finish(),
        }
    }
}