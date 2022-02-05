use crate::{advanced::{compute::{ComputeShader, ComputeShaderExecutionSettings}, shaderstorage::ShaderStorage, atomic::{AtomicGroupRead, AtomicGroup}}, basics::{readwrite::ReadBytes, transfer::Transfer, texture::Texture, shader::{info::{ShaderInfoQuerySettings, ShaderInfo}, Shader}}};

use super::ObjectID;

// A task that can be sent to the render thread, but we can also check if it has finished executing
pub enum PipelineTrackedTask {
    RunComputeShader(ObjectID<ComputeShader>, ComputeShaderExecutionSettings),
    TextureReadBytes(ObjectID<Texture>, Transfer<ReadBytes>),
    ShaderStorageReadBytes(ObjectID<ShaderStorage>, Transfer<ReadBytes>),
    AtomicGroupRead(ObjectID<AtomicGroup>, Transfer<AtomicGroupRead>),
    QueryShaderInfo(ObjectID<Shader>, ShaderInfoQuerySettings, Transfer<ShaderInfo>),
    QueryComputeShaderInfo(ObjectID<ComputeShader>, ShaderInfoQuerySettings, Transfer<ShaderInfo>),
}