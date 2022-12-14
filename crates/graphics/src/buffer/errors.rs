use thiserror::Error;

// Buffer creation error (only one really)
#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("The given buffer mode must not be BufferMode::Resizable if the length of the buffer is 0")]
    NotResizable
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

// Buffer error that is returned from each buffer command
#[derive(Error, Debug)]
pub enum BufferError {
    #[error("{0}")]
    InvalidMode(InvalidModeError),

    #[error("{0}")]
    InvalidUsage(InvalidUsageError),

    #[error("{0}")]
    Initializion(InitializationError),

    #[error(
        "Tried accessing slice of size {0} with range of size {1}"
    )]
    SliceLengthRangeMistmatch(usize, usize),

    #[error("The given range {0}..{1} is an invalid length for buffer with size {2}")]
    InvalidRangeSize(usize, usize, usize),
}