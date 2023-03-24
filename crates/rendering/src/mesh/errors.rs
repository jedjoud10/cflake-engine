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
pub enum MeshAttributeError {
    #[error("The given mip level was already mutably borrowed")]
    BorrowedMutably,

    #[error("The given mip level was already immutably borrowed")]
    BorrowedImmutably,

    #[error("The given mip level ({0}) is out of the mip levels within the texture ({1})")]
    OutOfRange(u8, u8)
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

    #[error("The position attribute buffer does not exist. Cannot create the AABB")]
    MissingPositionAttributeBuffer,
}
