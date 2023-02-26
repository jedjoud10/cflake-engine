use thiserror::Error;

#[derive(Error, Debug)]
pub enum BufferInitializationError {
    #[error("The given buffer mode must be BufferMode::Resizable if the slice is empty")]
    EmptySliceNotResizable,

    #[error("Given buffer variant type is invalid. Must be VERTEX, INDEX, STORAGE, UNIFORM, or INDIRECT")]
    InvalidVariantType,

    #[error("The given buffer usage contains the WRITE flag, but there isn't the COPY_DST flag")]
    WritableWithoutCopyDst,

    #[error("The given buffer usage contains the READ flag, but there isn't the COPY_SRC flag")]
    ReadableWithoutCopySrc,
}

#[derive(Error, Debug)]
pub enum BufferExtendError {
    #[error("Cannot extend the buffer since self.mode isn't BufferMode::Partial or BufferMode::Resizable")]
    IllegalLengthModify,

    #[error(
        "Cannot reallocate the buffer since self.mode isn't BufferMode::Resizable"
    )]
    IllegalReallocation,

    #[error("The buffer cannot be written since it's BufferUsages is not Write nor ReadWrite")]
    NonWritable,
}

#[derive(Error, Debug)]
pub enum BufferReadError {
    #[error("The given destination slice of length {0} (or offset of {1}) would overflow the buffer of length {2}")]
    InvalidLen(usize, usize, usize),

    #[error("The buffer cannot be read since it's BufferUsages is not Read nor ReadWrite")]
    NonReadable,
}

#[derive(Error, Debug)]
pub enum BufferWriteError {
    #[error("The given source slice of length {0} (or offset of {1}) would overflow the buffer of length {2}")]
    InvalidLen(usize, usize, usize),

    #[error("The buffer cannot be written since it's BufferUsages is not Write nor ReadWrite")]
    NonWritable,
}

#[derive(Error, Debug)]
pub enum BufferCopyError {
    #[error("The given source buffer does not have the COPY_SRC usage")]
    NonCopySrc,
    
    #[error("The destination buffer (self) does not have the COPY_DST usage")]
    NonCopyDst,

    #[error("The given length {0} (or offset of {1}) would overflow the destination buffer of length {2}")]
    InvalidSrcOverflow(usize, usize, usize),

    #[error("The given length {0} (or offset of {1}) would overflow the source buffer of length {2}")]
    InvalidDstOverflow(usize, usize, usize),
}

#[derive(Error, Debug)]
pub enum BufferClearError {
    #[error("Cannot clear the buffer since self.mode isn't BufferMode::Partial or BufferMode::Resizable")]
    IllegalLengthModify,
}

#[derive(Error, Debug)]
pub enum BufferNotMappableError {
    #[error("The buffer cannot be mapped (read) since it's BufferUsages is not Read nor ReadWrite")]
    AsView,

    #[error("The buffer cannot be mapped (for reading AND writing) since it's BufferUsages is not ReadWrite")]
    AsViewMut,

    #[error("Invalid range")]
    InvalidRange,
}
