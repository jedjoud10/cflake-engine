use std::rc::Rc;

use crate::basics::Texture;

// Some default uniforms that we will set
#[derive(Clone)]
pub enum Uniform {
    // Singles
    F32(f32),
    I32(i32),
    // Vectors
    Vec2F32(veclib::Vector2<f32>),
    Vec3F32(veclib::Vector3<f32>),
    Vec4F32(veclib::Vector4<f32>),
    Vec2I32(veclib::Vector2<i32>),
    Vec3I32(veclib::Vector3<i32>),
    Vec4I32(veclib::Vector4<i32>),
    Mat44F32(veclib::Matrix4x4<f32>),
    // Others
    Texture2D(Rc<Texture>, u32),
    Texture3D(Rc<Texture>, u32),
    Texture2DArray(Rc<Texture>, u32),
}

// Simple main OpenGL types
#[derive(Clone, Copy)]
pub enum DataType {
    // 8 bit
    UByte,
    Byte,
    // 16 bit
    UInt16,
    Int16,
    // 32 bit
    UInt32,
    Int32,
    // FP
    Float32,
}

// The texture format
#[derive(Clone, Copy, Debug)]
pub enum TextureFormat {
    // Red
    R8R,
    R16R,
    R8RS,
    R8I,
    R16I,
    R32I,
    // FP
    R16F,
    R32F,
    // Red Green
    RG8R,
    RG8RS,
    RG16R,
    RG8I,
    RG16I,
    RG32I,
    // FP
    RG16F,
    RG32F,
    // Red Green Blue
    RGB8R,
    RGB8RS,
    RGB16R,
    RGB8I,
    RGB16I,
    RGB32I,
    // FP
    RGB16F,
    RGB32F,
    // Red Green Blue Alpha
    RGBA8R,
    RGBA8RS,
    RGBA16R,
    RGBA8I,
    RGBA16I,
    RGBA32I,
    // FP
    RGBA16F,
    RGBA32F,

    // Custom
    DepthComponent16,
    DepthComponent24,
    DepthComponent32,
}

// Get the IDF from a simple TextureFormat and DataType
pub fn get_ifd(tf: TextureFormat, dt: DataType) -> (i32, u32, u32) {
    let internal_format = match tf {
        // Red
        TextureFormat::R8R => gl::R8,
        TextureFormat::R8RS => gl::R8_SNORM,
        TextureFormat::R16R => gl::R16,
        TextureFormat::R8I => gl::R8I,
        TextureFormat::R16I => gl::R16I,
        TextureFormat::R32I => gl::R32I,
        TextureFormat::R16F => gl::R16F,
        TextureFormat::R32F => gl::R32F,
        // Red Green
        TextureFormat::RG8R => gl::RG8,
        TextureFormat::RG8RS => gl::RG8_SNORM,
        TextureFormat::RG16R => gl::RG16,
        TextureFormat::RG8I => gl::RG8I,
        TextureFormat::RG16I => gl::RG16I,
        TextureFormat::RG32I => gl::RG32I,
        TextureFormat::RG16F => gl::RG16F,
        TextureFormat::RG32F => gl::RG32F,
        // Red Green Blue
        TextureFormat::RGB8R => gl::RGB8,
        TextureFormat::RGB8RS => gl::RGB8_SNORM,
        TextureFormat::RGB16R => gl::RGB16,
        TextureFormat::RGB8I => gl::RGB8I,
        TextureFormat::RGB16I => gl::RGB16I,
        TextureFormat::RGB32I => gl::RGB32I,
        TextureFormat::RGB16F => gl::RGB16F,
        TextureFormat::RGB32F => gl::RGB32F,
        // Red Green Blue Alpha
        TextureFormat::RGBA8R => gl::RGBA8,
        TextureFormat::RGBA8RS => gl::RGBA8_SNORM,
        TextureFormat::RGBA16R => gl::RGBA16,
        TextureFormat::RGBA8I => gl::RGBA8I,
        TextureFormat::RGBA16I => gl::RGBA16I,
        TextureFormat::RGBA32I => gl::RGBA32I,
        TextureFormat::RGBA16F => gl::RGBA16F,
        TextureFormat::RGBA32F => gl::RGBA32F,
        // Custom
        TextureFormat::DepthComponent16 => gl::DEPTH_COMPONENT16,
        TextureFormat::DepthComponent24 => gl::DEPTH_COMPONENT24,
        TextureFormat::DepthComponent32 => gl::DEPTH_COMPONENT32,
    };
    let format = match tf {
        TextureFormat::R8R
        | TextureFormat::R16R
        | TextureFormat::R8RS
        | TextureFormat::R8I
        | TextureFormat::R16I
        | TextureFormat::R32I
        | TextureFormat::R16F
        | TextureFormat::R32F => gl::RED,
        TextureFormat::RG8R
        | TextureFormat::RG16R
        | TextureFormat::RG8RS
        | TextureFormat::RG8I
        | TextureFormat::RG16I
        | TextureFormat::RG32I
        | TextureFormat::RG16F
        | TextureFormat::RG32F => gl::RG,
        TextureFormat::RGB8R
        | TextureFormat::RGB16R
        | TextureFormat::RGB8RS
        | TextureFormat::RGB8I
        | TextureFormat::RGB16I
        | TextureFormat::RGB32I
        | TextureFormat::RGB16F
        | TextureFormat::RGB32F => gl::RGB,
        TextureFormat::RGBA8R
        | TextureFormat::RGBA16R
        | TextureFormat::RGBA8RS
        | TextureFormat::RGBA8I
        | TextureFormat::RGBA16I
        | TextureFormat::RGBA32I
        | TextureFormat::RGBA16F
        | TextureFormat::RGBA32F => gl::RGBA,
        TextureFormat::DepthComponent16 | TextureFormat::DepthComponent24 | TextureFormat::DepthComponent32 => gl::DEPTH_COMPONENT,
    };
    let data_type = match dt {
        DataType::UByte => gl::UNSIGNED_BYTE,
        DataType::Byte => gl::BYTE,
        DataType::UInt16 => gl::UNSIGNED_SHORT,
        DataType::Int16 => gl::SHORT,
        DataType::UInt32 => gl::UNSIGNED_INT,
        DataType::Int32 => gl::INT,
        DataType::Float32 => gl::FLOAT,
    };
    (internal_format as i32, format as u32, data_type as u32)
}
