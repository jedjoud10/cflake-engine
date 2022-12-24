use thiserror::Error;

// Buffer creation error (only one really)
#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("The given buffer mode must be BufferMode::Resizable if the slice is empty")]
    EmptySliceNotResizable,
}


#[derive(Error, Debug)]
pub enum ExtendError {
    #[error("Missing change length permission (BufferMode::Partial or BufferMode::Resizable)")]
    IllegalLengthModify,

    #[error("Missing reallocation permission (BufferMode::Resizable)")]
    IllegalReallocation,
}


#[derive(Error, Debug)]
pub enum ReadError {
    #[error("The given destination slice of length {0} (or offset of {1}) would overflow the buffer of length {2}")]
    InvalidLen(usize, usize, usize)
}

#[derive(Error, Debug)]
pub enum WriteError {
    #[error("The given source slice of length {0} (or offset of {1}) would overflow the buffer of length {2}")]
    InvalidLen(usize, usize, usize)
}

#[derive(Error, Debug)]
pub enum CopyError {
    #[error("The given length {0} (or offset of {1}) would overflow the destination buffer of length {2}")]
    InvalidSrcOverflow(usize, usize, usize),

    #[error("The given length {0} (or offset of {1}) would overflow the source buffer of length {2}")]
    InvalidDstOverflow(usize, usize, usize)
}



// Buffer error that is returned from each buffer command
#[derive(Error, Debug)]
pub enum BufferError {
    #[error("{0}")]
    Initialization(InitializationError),

    #[error("{0}")]
    WriteError(WriteError),

    #[error("{0}")]
    ReadError(ReadError),

    #[error("{0}")]
    CopyError(CopyError),

    #[error("{0}")]
    ExtendError(ExtendError),

    #[error("The given buffer cannot be mapped to host memory")]
    NotMappable,
}
