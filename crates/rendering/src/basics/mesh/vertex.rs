use super::Vertices;

// A single vertex
pub struct Vertex {
    position: veclib::Vector3<f32>,
    normal: Option<veclib::Vector3<i8>>,
    tangent: Option<veclib::Vector4<i8>>,
    uv: veclib::Vector2<u8>,
    color: Option<veclib::Vector3<u8>>
}

// A vertex builder that helps us create a vertex
pub struct VertexBuilder<'a> {
    pub(crate) vertices: &'a mut Vertices,
}

impl<'a> VertexBuilder<'a> {
}
