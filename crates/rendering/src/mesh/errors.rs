use graphics::{
    BufferInitializationError, BufferNotMappableError,
};
use obj::ObjError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MeshImportError {
    #[error("{0}")]
    Initialization(MeshInitializationError),

    #[error("{0}")]
    ObjError(ObjError),
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
