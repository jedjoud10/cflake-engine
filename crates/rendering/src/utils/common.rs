use gl::{types::GLuint, FramebufferParameteri};

// Simple main OpenGL types
#[derive(Clone, Copy)]
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
    WriteOnceReadSometimes,
}

// How we will use a buffer
#[derive(Clone, Copy)]
pub struct UsageType {
    pub access: AccessType,
    pub frequency: UpdateFrequency,
    pub dynamic: bool,
}

impl Default for UsageType {
    fn default() -> Self {
        Self {
            access: AccessType::ClientToServer,
            frequency: UpdateFrequency::WriteManyReadMany,
            dynamic: true,
        }
    }
}

impl UsageType {
    // Convert this UsageType to a valid OpenGL enum
    pub fn convert(&self) -> GLuint {
        match self.access {
            AccessType::ClientToServer => match self.frequency {
                UpdateFrequency::WriteOnceReadMany => gl::STATIC_DRAW,
                UpdateFrequency::WriteManyReadMany => gl::DYNAMIC_DRAW,
                UpdateFrequency::WriteOnceReadSometimes => gl::STREAM_DRAW,
            },
            AccessType::ServerToClient => match self.frequency {
                UpdateFrequency::WriteOnceReadMany => gl::STATIC_READ,
                UpdateFrequency::WriteManyReadMany => gl::DYNAMIC_READ,
                UpdateFrequency::WriteOnceReadSometimes => gl::STREAM_READ,
            },
            AccessType::ServerToServer => match self.frequency {
                UpdateFrequency::WriteOnceReadMany => gl::STATIC_COPY,
                UpdateFrequency::WriteManyReadMany => gl::DYNAMIC_COPY,
                UpdateFrequency::WriteOnceReadSometimes => gl::STREAM_COPY,
            },
        }
    }
}
