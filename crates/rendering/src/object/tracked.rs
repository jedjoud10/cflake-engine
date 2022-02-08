use super::ObjectID;
use crate::{
    advanced::{
        atomic::{AtomicGroup, AtomicGroupRead},
        compute::{ComputeShader, ComputeShaderExecutionSettings},
        shader_storage::ShaderStorage,
    },
    basics::{
        readwrite::ReadBytes,
        shader::{
            info::{ShaderInfoQuerySettings, ShaderInfoRead},
        },
        texture::Texture,
        transfer::Transfer,
        uniforms::ShaderIDType,
    },
};

// A task that can be sent to the render thread, but we can also check if it has finished executing
pub enum TrackedTask {
    RunComputeShader(ObjectID<ComputeShader>, ComputeShaderExecutionSettings),
    TextureReadBytes(ObjectID<Texture>, Transfer<ReadBytes>),
    ShaderStorageReadBytes(ObjectID<ShaderStorage>, Transfer<ReadBytes>),
    AtomicGroupRead(ObjectID<AtomicGroup>, Transfer<AtomicGroupRead>),
    QueryShaderInfo(ShaderIDType, ShaderInfoQuerySettings, Transfer<ShaderInfoRead>),
}
