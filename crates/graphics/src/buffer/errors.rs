use thiserror::Error;

#[derive(Error, Debug)]
pub enum BufferInitializationError {
    #[error("The given buffer mode must be BufferMode::Resizable if the slice is empty")]
    EmptySliceNotResizable,

    #[error("Given buffer variant type is invalid. Must be VERTEX, INDEX, STORAGE, UNIFORM, or INDIRECT")]
    InvalidVariantType,
}

#[derive(Error, Debug)]
pub enum BufferExtendError {
    #[error("Cannot extend the buffer since self.mode isn't BufferMode::Partial or BufferMode::Resizable")]
    IllegalLengthModify,

    #[error(
        "Cannot reallocate the buffer since self.mode isn't BufferMode::Resizable"
    )]
    IllegalReallocation,

    #[error("The buffer cannot be written since it's BufferUsages does not contain the write flag")]
    NonWritable,
}

#[derive(Error, Debug)]
pub enum BufferReadError {
    #[error("The given destination slice of length {0} (or offset of {1}) would overflow the buffer of length {2}")]
    InvalidLen(usize, usize, usize),

    #[error("The buffer cannot be read since it's BufferUsages does not contain the read flag")]
    NonReadable,
}

#[derive(Error, Debug)]
pub enum BufferWriteError {
    #[error("The given source slice of length {0} (or offset of {1}) would overflow the buffer of length {2}")]
    InvalidLen(usize, usize, usize),

    #[error("The buffer cannot be written since it's BufferUsages does not contain the write flag")]
    NonWritable,
}

#[derive(Error, Debug)]
pub enum BufferCopyError {
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
    #[error("The buffer cannot be mapped (read) since it's BufferUsages does not contain the write flag")]
    NonReadable,

    #[error("The buffer cannot be mapped (write) since it's BufferUsages does not contain the write flag")]
    NonWritable,
}
