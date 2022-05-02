use std::num::NonZeroU32;
use gl::types::GLuint;

// Simple main OpenGL types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataType {
    // 8 bit
    U8,
    I8,
    // 16 bit
    U16,
    I16,
    // 32 bit
    U32,
    I32,
    // FP
    F32,
}

impl DataType {
    // Convert this data type to an OpenGL data type
    pub fn convert(&self) -> u32 {
        match self {
            DataType::U8 => gl::UNSIGNED_BYTE,
            DataType::I8 => gl::BYTE,
            DataType::U16 => gl::UNSIGNED_SHORT,
            DataType::I16 => gl::SHORT,
            DataType::U32 => gl::UNSIGNED_INT,
            DataType::I32 => gl::INT,
            DataType::F32 => gl::FLOAT,
        }
    }
}

// This hints the program how we might acces the data
#[derive(Clone, Copy)]
pub enum Access {
    Write, Read, ReadWrite, Copy,
}

// How frequently we will update the data of a buffer object
#[derive(Clone, Copy)]
pub enum Update {
    Static,
    Dynamic,
    Stream,
}


// Some hints that we give OpenGL that tell it how we might access a buffer
#[derive(Clone, Copy)]
pub struct BufferHints {
    // Normal hints
    pub access: Access,
    pub update: Update,

    // This would allow the buffer to resize to any length if it is enabled
    pub dynamic: bool,

    // This hints that buffer should be stored in client memory
    pub client: bool,


}

impl Default for BufferHints {
    fn default() -> Self {
        Self {
            access: Access::ReadWrite,
            update: Update::Stream,
            dynamic: true,
            client: false,
        }
    }
}

impl BufferHints {
    // Mutable storage buffer usage hints
    pub fn usage_hints(&self) -> GLuint {
        match (self.access, self.update) {
            (Access::Write, Update::Static) => gl::STATIC_DRAW,
            (Access::Write, Update::Dynamic) => gl::DYNAMIC_DRAW,
            (Access::Write, Update::Stream) => gl::STREAM_DRAW,
            (Access::Read, Update::Static) => gl::STATIC_READ,
            (Access::Read, Update::Dynamic) => gl::DYNAMIC_READ,
            (Access::Read, Update::Stream) => gl::STREAM_READ,
            (Access::ReadWrite, Update::Static) => gl::STATIC_DRAW,
            (Access::ReadWrite, Update::Dynamic) => gl::DYNAMIC_DRAW,
            (Access::ReadWrite, Update::Stream) => gl::STREAM_COPY,
            (Access::Copy, Update::Static) => gl::STATIC_COPY,
            (Access::Copy, Update::Dynamic) => gl::DYNAMIC_COPY,
            (Access::Copy, Update::Stream) => gl::STREAM_COPY,
        }
    }

    // Immutable storage buffer usage hints
    pub fn mapped_access_bit(&self) -> GLuint {
        let base = match self.access {
            Access::Write => gl::MAP_WRITE_BIT,
            Access::Read => gl::MAP_READ_BIT,
            Access::ReadWrite => gl::MAP_WRITE_BIT | gl::MAP_READ_BIT,
            Access::Copy => 0,
        };

        let client = if self.client { gl::CLIENT_STORAGE_BIT } else { 0 };
        base | client
    }

    // Check if we can read from the buffer
    pub fn readable(&self) -> bool {
        match self.access {
            Access::Read | Access::ReadWrite => true,
            _ => false
        }
    }

    // Check if we write to the buffer
    pub fn writable(&self) -> bool {
        match self.access {
            Access::Write | Access::ReadWrite => true,
            _ => false
        }
    }
}
