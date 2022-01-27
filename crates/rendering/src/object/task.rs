use std::sync::{Arc, Mutex};

use super::{ObjectID, PipelineObject, TrackedTaskID};
use crate::{
    advanced::{compute::{ComputeShader, ComputeShaderExecutionSettings}, atomic::{AtomicGroup, AtomicGroupRead}},
    basics::{
        material::Material,
        model::Model,
        renderer::Renderer,
        shader::Shader,
        texture::{Texture, TextureReadBytes, TextureWriteBytes},
        Buildable, transfer::Transfer,
    },
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
    // Update tasks
    UpdateRendererMatrix(ObjectID<Renderer>, veclib::Matrix4x4<f32>),
    UpdateTextureDimensions(ObjectID<Texture>, crate::basics::texture::TextureType),
    UpdateCamera(Camera),
    ReadAtomicGroup(ObjectID<AtomicGroup>, Transfer<AtomicGroupRead>),
    // Specific pipeline tasks
    SetWindowDimension(veclib::Vector2<u16>),
    SetWindowFocusState(bool),
}

// A task that can be sent to the render thread, but we can also check if it has finished executing
pub enum PipelineTrackedTask {
    RunComputeShader(ObjectID<ComputeShader>, ComputeShaderExecutionSettings),
    TextureReadBytes(ObjectID<Texture>, Transfer<TextureReadBytes>),
    TextureWriteBytes(ObjectID<Texture>, Transfer<TextureWriteBytes>),
}

// Bruh
pub enum PipelineTaskCombination {
    // Normal tasks
    Single(PipelineTask),
    SingleReqTracked(PipelineTask, TrackedTaskID),
    Batch(Vec<PipelineTask>),

    // Tracking task
    SingleTracked(PipelineTrackedTask, TrackedTaskID, Option<TrackedTaskID>),
    SingleTrackedFinalizer(TrackedTaskID, Vec<TrackedTaskID>), 
    // Compute Shader (Self: 0)
    // Finalizer (Self: 1, Requires: [0])
}
