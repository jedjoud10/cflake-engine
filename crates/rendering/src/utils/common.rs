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

// How we will access a buffer object
#[derive(Clone, Copy)]
pub enum AccessType {
    ClientToServer,
    ServerToClient,
    ServerToServer,
}

// How frequently we will update the data of a buffer object
#[derive(Clone, Copy)]
pub enum UpdateFrequency {
    WriteOnceReadMany,
    WriteManyReadMany,
    WriteSometimesReadMany,
}

// Some hints that we give OpenGL that tell it how we might access a buffer
#[derive(Clone, Copy)]
pub struct BufferHints {
    // How we will access the buffer on the GPU / CPU side
    pub access: AccessType,

    // How many times we will update the buffer per frame
    // TODO: Actually think abt this
    pub frequency: UpdateFrequency,

    // This would allow the buffer to resize to any length if it is enabled
    pub dynamic: bool,
}

impl Default for BufferHints {
    fn default() -> Self {
        Self {
            access: AccessType::ClientToServer,
            frequency: UpdateFrequency::WriteManyReadMany,
            dynamic: true,
        }
    }
}

impl BufferHints {
    // Get the OpenGL basic buffer flag hints from self
    pub fn into_access_hints(&self) -> GLuint {
        match self.access {
            AccessType::ClientToServer => match self.frequency {
                UpdateFrequency::WriteOnceReadMany => gl::STATIC_DRAW,
                UpdateFrequency::WriteManyReadMany => gl::STREAM_DRAW,
                UpdateFrequency::WriteSometimesReadMany => gl::DYNAMIC_DRAW,
            },
            AccessType::ServerToClient => match self.frequency {
                UpdateFrequency::WriteOnceReadMany => gl::STATIC_READ,
                UpdateFrequency::WriteManyReadMany => gl::STREAM_READ,
                UpdateFrequency::WriteSometimesReadMany => gl::DYNAMIC_READ,
            },
            AccessType::ServerToServer => match self.frequency {
                UpdateFrequency::WriteOnceReadMany => gl::STATIC_COPY,
                UpdateFrequency::WriteManyReadMany => gl::STREAM_COPY,
                UpdateFrequency::WriteSometimesReadMany => gl::DYNAMIC_COPY,
            },
        }
    }

    // Get the OpenGL mapped buffer flag hints from self
    pub fn into_mapped_buffer_hints(&self) -> u32 {
        match self.access {
            AccessType::ClientToServer => gl::DYNAMIC_STORAGE_BIT | gl::MAP_WRITE_BIT,
            AccessType::ServerToClient => gl::DYNAMIC_STORAGE_BIT | gl::MAP_READ_BIT,
            AccessType::ServerToServer => gl::MAP_READ_BIT,
        }
    }
}
