use thiserror::Error;

#[derive(Error, Debug)]
pub enum BufferInitializationError {
    #[error("The given buffer mode must be BufferMode::Resizable if the slice is empty")]
    EmptySliceNotResizable,

    #[error("The given buffer usage contains the WRITE flag, but there isn't the COPY_DST flag")]
    WritableWithoutCopyDst,

    #[error("The given buffer usage contains the READ flag, but there isn't the COPY_SRC flag")]
    ReadableWithoutCopySrc,

    #[error("The given buffer mode is Resizable, but there isn't the COPY_SRC flag in the usages")]
    ResizableWithoutCopySrc,

    #[error(
        "Cannot create a buffer with no usages or without valid type"
    )]
    UnkownBufferUsageOrType,
}

#[derive(Error, Debug)]
pub enum BufferExtendError {
    #[error("Cannot extend the buffer since self.mode isn't BufferMode::Partial or BufferMode::Resizable")]
    IllegalLengthModify,

    #[error(
        "Cannot reallocate the buffer since self.mode isn't BufferMode::Resizable"
    )]
    IllegalReallocation,

    #[error("The buffer cannot be written since it's BufferUsages do not contain the WRITE flag")]
    NonWritable,
}

#[derive(Error, Debug)]
pub enum BufferReadError {
    #[error("The given destination slice of length {0} (or offset of {1}) would overflow the buffer of length {2}")]
    InvalidLen(usize, usize, usize),

    #[error("The buffer cannot be read since it's BufferUsages do not contain the READ flag")]
    NonReadable,
}

#[derive(Error, Debug)]
pub enum BufferWriteError {
    #[error("The given source slice of length {0} (or offset of {1}) would overflow the buffer of length {2}")]
    InvalidLen(usize, usize, usize),

    #[error("The buffer cannot be written since it's BufferUsages do not contain the WRITE flag")]
    NonWritable,
}

#[derive(Error, Debug)]
pub enum BufferCopyError {
    #[error(
        "The given source buffer does not have the COPY_SRC usage"
    )]
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
pub enum BufferSplatError {
    #[error("The buffer cannot be splatted since it's BufferUsages do not contain the WRITE flag")]
    NonWritable,

    #[error(
        "The given range is invalid for buffer with {0} elements"
    )]
    InvalidRange(usize),
}

#[derive(Error, Debug)]
pub enum BufferNotMappableError {
    #[error("The buffer cannot be mapped (read) since it's BufferUsages do not contain the WRITE flag")]
    AsView,

    #[error("The buffer cannot be mapped (for reading AND writing) since it's BufferUsages do not contain the WRITE and READ flags")]
    AsViewMut,

    #[error(
        "The given range is invalid for buffer with {0} elements"
    )]
    InvalidRange(usize),
}
