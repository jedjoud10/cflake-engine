use thiserror::Error;

// Buffer creation error (only one really)
#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("The given buffer mode must not be BufferMode::Resizable if the length of the buffer is 0")]
    NotResizable,

    #[error("The given buffer mode is BufferMode::Resizable, but BufferUsage.device_read is false")]
    ResizableMissingDeviceRead,

    #[error("The given buffer mode is BufferMode::Resizable, but BufferUsage.device_write is false")]
    ResizableMissingDeviceWrite,

    #[error("The stride of T is zero. Currently, buffers cannot support zero-sized types")]
    ZeroSizedStride
}

// Buffer invalid mode error if we have invalid permissions
#[derive(Error, Debug)]
pub enum InvalidModeError {
    #[error("Missing change length permission (BufferMode::Resizable or BufferMode::Partial)")]
    IllegalChangeLength,

    #[error(
        "Missing reallocation permission (BufferMode::Resizable)"
    )]
    IllegalReallocation,
}

// Buffer invalid usage error if we have invalid permissions
#[derive(Error, Debug)]
pub enum InvalidUsageError {
    #[error("Cannot read from buffer since BufferUsages.host_read is false")]
    IllegalHostRead,

    #[error("Cannot write to buffer since BufferUsages.host_write is false")]
    IllegalHostWrite,
}

#[derive(Error, Debug)]
#[error("The given range {0}..{1} is an invalid length for buffer with size {2}")]
pub struct InvalidRangeSizeError(pub usize, pub usize, pub usize);

// Buffer error that is returned from each buffer command
#[derive(Error, Debug)]
pub enum BufferError {
    #[error("{0}")]
    Initialization(InitializationError),

    #[error("{0}")]
    InvalidSrcUsage(InvalidUsageError),

    #[error("{0}")]
    InvalidSrcMode(InvalidModeError),

    #[error("{0}")]
    InvalidSrcRange(InvalidRangeSizeError),

    #[error("{0}")]
    InvalidDstUsage(InvalidUsageError),

    #[error("{0}")]
    InvalidDstMode(InvalidModeError),

    #[error("{0}")]
    InvalidDstRange(InvalidRangeSizeError),
}
