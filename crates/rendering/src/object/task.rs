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