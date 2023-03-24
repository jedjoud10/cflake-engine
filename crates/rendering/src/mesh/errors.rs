use std::cell::{BorrowMutError, BorrowError};

use graphics::{BufferInitializationError, BufferNotMappableError};
use obj::ObjError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MeshImportError {
    #[error("{0}")]
    Initialization(MeshInitializationError),

    #[error("{0}")]
    ObjError(ObjError),
}

#[derive(Debug, Error)]
pub enum AttributeError {
    #[error("{0}")]
    BorrowError(BorrowError),

    #[error("{0}")]
    BorrowMutError(BorrowMutError),

    #[error("The given attribute does not exist on the mesh")]
    MissingAttribute,
}

#[derive(Error, Debug)]
pub enum MeshInitializationError {
    #[error("{0}")]
    AttributeBufferInitialization(BufferInitializationError),
}

#[derive(Debug, Error)]
pub enum MeshAabbComputeError {
    #[error(
        "Positions attribute buffer is empty, cannot calculate AABB"
    )]
    EmptyPositionAttributeBuffer,

    #[error(
        "The position buffer cannot be read from the host (CPU): {0}"
    )]
    NotHostMapped(BufferNotMappableError),

    #[error("{0}")]
    AttributeBuffer(AttributeError),
}
