use super::ObjectID;
use crate::{
    advanced::{
        compute::{ComputeShader, ComputeShaderExecutionSettings},
        shader_storage::ShaderStorage, atomic::AtomicGroup,
    },
    basics::{
        shader::info::{ShaderInfoQuerySettings, ShaderInfoRead},
        texture::Texture,
        uniforms::ShaderIDType, buffer_operation::BufferOperation,
    },
};

// A task that can be sent to the render thread, but we can also check if it has finished executing
pub enum TrackedTask {
    RunComputeShader(ObjectID<ComputeShader>, ComputeShaderExecutionSettings),
    TextureOp(ObjectID<Texture>, BufferOperation),
    ShaderStorageOp(ObjectID<ShaderStorage>, BufferOperation),
    AtomicGroupOp(ObjectID<AtomicGroup>, BufferOperation),
    QueryShaderInfo(ShaderIDType, ShaderInfoQuerySettings, ShaderInfoRead),
}
