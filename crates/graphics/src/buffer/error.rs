use thiserror::Error;

// Buffer invalid mode error if we have invalid permissions
#[derive(Error, Debug)]
pub enum InvalidModeError {
    #[error("Missing change length permission (BufferMode::Resizable or BufferMode::Partial)")]
    IllegalChangeLength,

    #[error("Missing reallocation permission (BufferMode::Resizable)")]
    IllegalReallocation,
}

// Buffer invalid usage error if we have invalid permissions
#[derive(Error, Debug)]
pub enum InvalidUsageError {
    #[error("Cannot read from buffer since BufferUsages:host_read is false")]
    IllegalHostRead,

    #[error("Cannot write to buffer since BufferUsages:host_write is false")]
    IllegalHostWrite
}

// Buffer error that is returned from each buffer command
#[derive(Error, Debug)]
pub enum BufferError {
    #[error("{0}")]
    InvalidMode(InvalidModeError),

    #[error("{0}")]
    InvalidUsage(InvalidUsageError),

    #[error("Tried accessing slice of size {0} with range of size {1}")]
    SliceLengthRangeMistmatch(usize, usize),

    #[error("Tried accessing buffer of size {0} with range of size {1}")]
    BufferLengthMismatch(usize, usize)
}