use std::sync::{Arc, Mutex};

use super::{ObjectID, PipelineObject, ReservedTrackedTaskID};
use crate::{
    advanced::{
        atomic::{AtomicGroup, AtomicGroupRead},
        compute::{ComputeShader, ComputeShaderExecutionSettings},
        shaderstorage::ShaderStorage,
    },
    basics::{material::Material, model::Model, readwrite::ReadBytes, renderer::Renderer, shader::{Shader, info::{ShaderInfo, ShaderInfoQuerySettings}}, texture::Texture, transfer::Transfer, Buildable},
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
    CreateModel(ObjectBuildingTask<Model>),
    CreateRenderer(ObjectBuildingTask<Renderer>),
    CreateAtomicGroup(ObjectBuildingTask<AtomicGroup>),
    CreateShaderStorage(ObjectBuildingTask<ShaderStorage>),
    // Update tasks
    UpdateRendererMatrix(ObjectID<Renderer>, veclib::Matrix4x4<f32>),
    UpdateTextureDimensions(ObjectID<Texture>, crate::basics::texture::TextureType),
    UpdateCamera(Camera),
    // Specific pipeline tasks
    SetWindowDimension(veclib::Vector2<u16>),
    SetWindowFocusState(bool),
}

// A task that can be sent to the render thread, but we can also check if it has finished executing
pub enum PipelineTrackedTask {
    RunComputeShader(ObjectID<ComputeShader>, ComputeShaderExecutionSettings),
    TextureReadBytes(ObjectID<Texture>, Transfer<ReadBytes>),
    ShaderStorageReadBytes(ObjectID<ShaderStorage>, Transfer<ReadBytes>),
    AtomicGroupRead(ObjectID<AtomicGroup>, Transfer<AtomicGroupRead>),
    QueryShaderInfo(ObjectID<Shader>, ShaderInfoQuerySettings, Transfer<ShaderInfo>),
    Test,
}

// Bruh
pub enum PipelineTaskCombination {
    // Normal tasks
    Single(PipelineTask),
    SingleReqTracked(PipelineTask, ReservedTrackedTaskID),
    Batch(Vec<PipelineTask>),

    // Tracking task
    SingleTracked(PipelineTrackedTask, ReservedTrackedTaskID, Option<ReservedTrackedTaskID>),
}
