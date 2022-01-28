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
