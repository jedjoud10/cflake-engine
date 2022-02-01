// Simple main OpenGL types
#[derive(Clone, Copy, Debug)]
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
    Write,
    Read,
    Pass,
}
// How frequently we will update the data of a buffer object
#[derive(Clone, Copy)]
pub enum UpdateFrequency {
    Static,
    Dynamic,
    Stream,
}

// How we will use a buffer
#[derive(Clone, Copy)]
pub struct UsageType {
    pub access: AccessType,
    pub frequency: UpdateFrequency,
}

impl UsageType {
    // Convert this UsageType to a valid OpenGL enum
    pub fn convert(&self) -> u32 {
        match self.access {
            AccessType::Write => match self.frequency {
                UpdateFrequency::Static => gl::STATIC_DRAW,
                UpdateFrequency::Dynamic => gl::DYNAMIC_DRAW,
                UpdateFrequency::Stream => gl::STREAM_DRAW,
            },
            AccessType::Read => match self.frequency {
                UpdateFrequency::Static => gl::STATIC_READ,
                UpdateFrequency::Dynamic => gl::DYNAMIC_READ,
                UpdateFrequency::Stream => gl::STREAM_READ,
            },
            AccessType::Pass => match self.frequency {
                UpdateFrequency::Static => gl::STATIC_COPY,
                UpdateFrequency::Dynamic => gl::DYNAMIC_COPY,
                UpdateFrequency::Stream => gl::STREAM_COPY,
            },
        }
    }
}
