use crate::pipeline::object::*;
use crate::{GPUObjectID, TextureShaderAccessType};

// Some default uniforms that we will set
#[derive(Clone)]
pub enum Uniform {
    // Singles
    Bool(bool),
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
    Texture1D(GPUObjectID, u32),
    Texture2D(GPUObjectID, u32),
    Texture3D(GPUObjectID, u32),
    Texture2DArray(GPUObjectID, u32),
    // Compute sheit
    Image2D(GPUObjectID, TextureShaderAccessType),
    Image3D(GPUObjectID, TextureShaderAccessType),
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
