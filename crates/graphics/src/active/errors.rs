use thiserror::Error;

use crate::ModuleVisibility;

#[derive(Error, Debug)]
pub enum SetBindResourceError<'a> {
    #[error("The bind resource '{name}' at bind group '{group}' was not defined in the shader layout")]
    ResourceNotDefined { name: &'a str, group: u32 },

    #[error("The given buffer at '{name}' has a different type [size = {inputted}] than the one defined in the shader layout [size = {defined}]")]
    BufferDifferentType {
        name: &'a str,
        defined: usize,
        inputted: usize,
    },
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
