use std::ptr::NonNull;
use super::VertexSet;


// Marker vertex attributes that we will use to fetch/add vertices
pub struct Position;
pub struct Tangent;
pub struct Normal;
pub struct TexCoord;
pub struct Color;


// An attribute that contains complementary vertex data
pub trait Attribute {
    // The underlying data that will be given to the user
    type Type;

    // Get the base pointer for this specific attribute
    fn base(set: &VertexSet) -> NonNull<Self::Type>;
}

impl Attribute for Position {
    type Type = vek::Vec3<f32>;

    fn base(set: &VertexSet) -> NonNull<Self::Type> {
        set.positions
    }
}

impl Attribute for Normal {
    type Type = vek::Vec3<f32>;

    fn base(set: &VertexSet) -> NonNull<Self::Type> {
        set.normals
    }
}

impl Attribute for Tangent {
    type Type = vek::Vec4<f32>;

    fn base(set: &VertexSet) -> NonNull<Self::Type> {
        set.tangents
    }
}

impl Attribute for TexCoord {
    type Type = vek::Vec2<u8>;

    fn base(set: &VertexSet) -> NonNull<Self::Type> {
        set.uvs
    }
}

impl Attribute for Color {
    type Type = vek::Rgb<u8>;

    fn base(set: &VertexSet) -> NonNull<Self::Type> {
        set.colors
    }
}

// A vertex layout is simply a trait that is implement for tuples of multiple attribute pointers