use thiserror::Error;
use crate::ModuleVisibility;

#[derive(Error, Debug)]
pub enum SetBindGroupError {
    #[error("The given bind group index {current} is greater than the adapter's supported bind group index {supported}")]
    BindGroupAdapterIndexInvalid {
        current: u32,
        supported: u32
    },

    #[error("The bind resource '{0}' was not set")]
    MissingResource(String),
}

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
pub enum SetBufferError {
    #[error("The given range is invalid for buffer with {0} elements")]
    InvalidRange(usize),

    #[error("The given storage buffer does not contain the STORAGE usage")]
    MissingStorageUsage,
}

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
pub enum DispatchError {
    #[error("Missing valid bind group for index {0}")]
    MissingValidBindGroup(u32),

    #[error("Given indirect buffer element index overruns the given indirect buffer")]
    InvalidIndirectIndex,
}

#[derive(Error, Debug)]
pub enum DrawError {
    #[error("Missing valid bind group for index {0}")]
    MissingValidBindGroup(u32),

    #[error("Missing vertex buffer for slot {0}")]
    MissingVertexBuffer(u32),

    #[error("Given indirect buffer element index overruns the given indirect buffer")]
    InvalidIndirectIndex,
}

#[derive(Error, Debug)]
pub enum DrawIndexedError {
    #[error("Missing valid bind group for index {0}")]
    MissingValidBindGroup(u32),

    #[error("Missing vertex buffer for slot {0}")]
    MissingVertexBuffer(u32),

    #[error("Missing index buffer")]
    MissingIndexBuffer,

    #[error("Given indirect buffer element index overruns the given indirect buffer")]
    InvalidIndirectIndex,
}