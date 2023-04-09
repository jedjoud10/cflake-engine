use thiserror::Error;

use crate::ModuleVisibility;

#[derive(Error, Debug)]
pub enum SetBindResourceError<'a> {
    #[error("The bind resource '{name}' at bind group '{group}' was not defined in the shader layout")]
    ResourceNotDefined { name: &'a str, group: u32 },

    #[error("{0}")]
    SetTexture(SetTextureError),

    #[error("{0}")]
    SetSampler(SetSamplerError),

    #[error("{0}")]
    SetBuffer(SetBufferError),

    #[error(
        "The given range is invalid for buffer with {0} elements"
    )]
    InvalidBufferRange(usize),
}

#[derive(Error, Debug)]
pub enum SetVertexBufferError {
    #[error(
        "The given range is invalid for buffer with {0} elements"
    )]
    InvalidRange(usize),

    #[error("There isn't a vertex buffer layout for slot {0}")]
    InvalidSlot(u32),

    #[error("The buffer Pod type for slot {0} does not match with the given one")]
    InvalidVertexInfo(u32),
}

#[derive(Error, Debug)]
pub enum SetIndexBufferError {
    #[error(
        "The given range is invalid for buffer with {0} elements"
    )]
    InvalidRange(usize),
}

#[derive(Error, Debug)]
pub enum SetPushConstantsError {}

#[derive(Error, Debug)]
pub enum SetTextureError {
    #[error("The given sampled texture does not contain the SAMPLE usage")]
    MissingSampleUsage,

    #[error("The given storage texture does not contain the STORAGE usage")]
    MissingStorageUsage,
}

#[derive(Error, Debug)]
pub enum SetSamplerError {}

#[derive(Error, Debug)]
pub enum SetBufferError {}

#[derive(Error, Debug)]
pub enum PushConstantBytesError {
    #[error("No bytes were defined to be pushed")]
    NoBytes,

    #[error("The given byte offset or byte size are too large and would overflow the defined push constant layout size")]
    OffsetOrSizeIsTooLarge,

    #[error("Tried setting a push constant with visibility {0:?}, but the current shader {1:?} does not support it")]
    VisibilityNotValid(ModuleVisibility, ModuleVisibility),
}

#[derive(Error, Debug)]
pub enum SetBindGroupError {}
