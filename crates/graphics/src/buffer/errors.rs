use thiserror::Error;

// Buffer creation error (only one really)
#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("The given slice is emtpy")]
    ZeroSizedSlice,

    #[error("The stride of T is zero. Currently, buffers cannot support zero-sized types")]
    ZeroSizedStride
}

// Buffer invalid mode error if we have invalid permissions
#[derive(Error, Debug)]
pub enum InvalidModeError {
    #[error("Missing change length permission (BufferMode::Partial)")]
    IllegalLengthModify,
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
    InvalidUsage(InvalidUsageError),

    #[error("{0}")]
    InvalidMode(InvalidModeError),

    #[error("{0}")]
    InvalidRange(InvalidRangeSizeError),

    // Only used in the copy command
    #[error("{0}")]
    InvalidDstUsage(InvalidUsageError),

    // Only used in the copy command
    #[error("{0}")]
    InvalidDstMode(InvalidModeError),

    // Only used in the copy command
    #[error("{0}")]
    InvalidDstRange(InvalidRangeSizeError),
}
