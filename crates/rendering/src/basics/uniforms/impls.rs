use crate::{object::ObjectID, Texture, TextureShaderAccessType};

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
    Texture1D(ObjectID<Texture>, u32),
    Texture2D(ObjectID<Texture>, u32),
    Texture3D(ObjectID<Texture>, u32),
    Texture2DArray(ObjectID<Texture>, u32),
    // Compute sheit
    Image2D(ObjectID<Texture>, TextureShaderAccessType),
    Image3D(ObjectID<Texture>, TextureShaderAccessType),
}

// A uniform trait that can be implemented on anything that can be converted into a uniform
pub trait Uniformable {
    // Get the enum
    fn get_uniform(self) -> Uniform;
}