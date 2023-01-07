use graphics::{BufferUsage, BufferInitializationError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MeshImportError {
    #[error("{0}")]
    Initialization(MeshInitializationError)
}

#[derive(Error, Debug)]
pub enum MeshInitializationError {
    #[error("{0}")]
    AttributeBufferInitialization(BufferInitializationError),
}

#[derive(Debug, Error)]
#[error("The position buffer cannot be read from the host (CPU). Current buffer usage is {0:?}")]
pub struct MeshAabbComputeError(BufferUsage);