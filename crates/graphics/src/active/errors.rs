use thiserror::Error;

use crate::ModuleVisibility;

// Errors that might get returned whenever we try setting a resource
// Most of the validation checking is done when the shader is created, using BindLayout
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
pub enum SetPushConstantsError {
    #[error("Missing push constant range ({0}..{1}) with visibility {2:?}")]
    MissingRange(usize, usize, ModuleVisibility),

    #[error("The given range range ({0}..{1}) is greater than minimum required size (128)")]
    GreaterThanMinLimit(usize, usize),
}

#[derive(Error, Debug)]
pub enum SetBindGroupError {}
