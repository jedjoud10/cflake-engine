use crate::{
    advanced::{atomic::AtomicGroup, shader_storage::ShaderStorage},
    basics::texture::{Texture, TextureAccessType},
    object::ObjectID,
};

// Some default uniforms that we will set
#[derive(Clone)]
pub enum Uniform {
    // These are types that are in Unsized Vectors, vectors that could be a single value, or 2, or 3, or 4
    Bool(veclib::UnsizedVector<bool>),
    I32(veclib::UnsizedVector<i32>),
    F32(veclib::UnsizedVector<f32>),
    F64(veclib::UnsizedVector<f64>),
    // Matrices
    Mat44F32(veclib::Matrix4x4<f32>),
    // Others
    Texture(ObjectID<Texture>, u32),
    // Compute sheit
    Image(ObjectID<Texture>, TextureAccessType),
    CounterGroup(ObjectID<AtomicGroup>, u32),
    ShaderStorage(ObjectID<ShaderStorage>, u32),
}
