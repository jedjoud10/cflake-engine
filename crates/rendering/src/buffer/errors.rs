use super::{MapAccess, BufferMode};

// Error that gets thrown whenever the user fucks up something when dealing with the buffer
pub enum BufferError {
    // Buffer initialization errors
    EmptyStaticInit,
    EmptyDynamicInit, 

    // Buffer mapping errors
    MapPermissionsInvalidRead(MapAccess, BufferMode),
    MapPermissionsInvalidWrite(MapAccess, BufferMode),

    // Buffer read to slice error
    ReadToSliceInvalidLen(usize, usize),

    // Buffer write from slice errors
    WriteFromSliceInvalidLen(usize, usize),
    NotResizable,
    WriteStatic,

    // Buffer copy from / copy to
    CopyFromInvalidLen(usize, usize),
    CopyToInvalidLen(usize, usize),
}

impl std::fmt::Debug for BufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BufferError::EmptyStaticInit => write!(f, "Tried to initialize static buffer with no elements"),
            BufferError::EmptyDynamicInit => write!(f, "Tried to initialize dynamic buffer with no elements"),
            BufferError::ReadToSliceInvalidLen(buffer, slice) => write!(f, "Could not fill slice with buffer data. Length mismatch; slice length: {slice}, buffer length: {buffer}"),
            BufferError::MapPermissionsInvalidRead(required, mode) => write!(f, "Insufficient map access permissions for reading; required access: {:?}, buffer mode: {:?}", required, mode),
            BufferError::MapPermissionsInvalidWrite(required, mode) => write!(f, "Insufficient map access permissions for writing; required access: {:?}, buffer mode: {:?}", required, mode),
            BufferError::WriteFromSliceInvalidLen(buffer, slice) => write!(f, "Coud not fill buffer with slice data. Length mismatch; slice length: {slice}, buffer length: {buffer}"),
            BufferError::WriteStatic => write!(f, "Cannot mutate/write into static buffer"),
            BufferError::CopyFromInvalidLen(buffer, other) => write!(f, "Could not copy data from 'other' buffer. Length mismatch; current buffer byte count: {buffer}, other buffer byte count: {other}"),
            BufferError::CopyToInvalidLen(buffer, other) => write!(f, "Could not copy data into 'other' buffer. Length mismatch; current buffer byte count: {buffer}, other buffer byte count: {other}"),
            BufferError::NotResizable => write!(f, "Cannot resize the current buffer; buffer mode is invalid"),
        }
    }
}

impl std::fmt::Display for BufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}