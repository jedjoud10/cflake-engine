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
    #[error("The buffer is not readable at all since BufferUsages.host_read is false and BufferUsages.device_read is false")]
    IllegalRead,

    #[error("The buffer is not writable at all since BufferUsages.host_write is false and BufferUsages.device_write is false")]
    IllegalWrite,

    #[error("Cannot read from buffer since BufferUsages.host_read is false")]
    IllegalHostRead,

    #[error("Cannot write to buffer since BufferUsages.host_write is false")]
    IllegalHostWrite,

    #[error("Cannot read from buffer on the GPU since BufferUsages.device_read is false")]
    IllegalDeviceRead,

    #[error("Cannot write to buffer on the GPU since BufferUsages.device_write is false")]
    IllegalDeviceWrite,
}

#[derive(Error, Debug)]
#[error("The given range {0}..{1} is an invalid length for buffer with size {2}")]
pub struct InvalidRangeSizeError(pub usize, pub usize, pub usize);

// Splat range command error
#[derive(Error, Debug)]
pub enum SplatRangeError {
    #[error("{0}")]
    InvalidRangeSize(InvalidRangeSizeError),

    #[error("{0}")]
    InvalidUsage(InvalidUsageError),

    #[error("{0}")]
    InvalidMode(InvalidModeError),
}

// Extend from iter command error
#[derive(Error, Debug)]
pub enum ExtendFromIterError {
    #[error("{0}")]
    InvalidUsage(InvalidUsageError),

    #[error("{0}")]
    InvalidMode(InvalidModeError),

    #[error("{0}")]
    InvalidRangeSize(InvalidRangeSizeError),
}

// Write range command error
#[derive(Error, Debug)]
pub enum WriteRangeError {
    #[error("{0}")]
    InvalidUsage(InvalidUsageError),

    #[error("{0}")]
    InvalidRangeSize(InvalidRangeSizeError),

    #[error("a")]
    SliceLengthMismatch(),
}

// Read range command error
#[derive(Error, Debug)]
pub enum ReadRangeError {
    #[error("{0}")]
    InvalidUsage(InvalidUsageError),

    #[error("{0}")]
    InvalidRangeSize(InvalidRangeSizeError),

    #[error("a")]
    SliceLengthMismatch(),
}

// Clear command error
#[derive(Error, Debug)]
#[error("{0}")]
pub struct ClearError(InvalidUsageError);

// Copy range from error
#[derive(Error, Debug)]
pub enum CopyRangeFromError {
    #[error("{0}")]
    InvalidSrcUsage(InvalidUsageError),

    #[error("{0}")]
    InvalidDstUsage(InvalidUsageError),

    #[error("{0}")]
    InvalidSrcRangeSize(InvalidRangeSizeError),

    #[error("{0}")]
    InvalidDstRangeSize(InvalidRangeSizeError),
}

// Buffer error that is returned from each buffer command
#[derive(Error, Debug)]
pub enum BufferError {
    #[error("{0}")]
    Initialization(InitializationError),

    #[error("{0}")]
    SplatRange(SplatRangeError),

    #[error("{0}")]
    ExtendFromIter(ExtendFromIterError),

    #[error("{0}")]
    WriteRange(WriteRangeError),

    #[error("{0}")]
    ReadRange(ReadRangeError),

    #[error("{0}")]
    Clear(ClearError),

    #[error("{0}")]
    CopyRangeFrom(CopyRangeFromError),
}
