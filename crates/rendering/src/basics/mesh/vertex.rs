use super::Vertices;

// A single vertex
pub struct Vertex {}

// A vertex builder that helps us create a vertex
pub struct VertexBuilder<'a> {
    pub(crate) vertices: &'a mut Vertices,
}

impl<'a> VertexBuilder<'a> {
    // Le builder pattern
    pub fn with_position(self, position: veclib::Vector3<f32>) -> Self {
        self.vertices.positions.push(position);
        self
    }
    pub fn with_normal(self, normal: veclib::Vector3<i8>) -> Self {
        self.vertices.normals.push(normal);
        self
    }
    pub fn with_tangent(self, tangent: veclib::Vector4<i8>) -> Self {
        self.vertices.tangents.push(tangent);
        self
    }
    pub fn with_uv(self, uv: veclib::Vector2<u8>) -> Self {
        self.vertices.uvs.push(uv);
        self
    }
    pub fn with_color(self, color: veclib::Vector3<u8>) -> Self {
        self.vertices.colors.push(color);
        self
    }
}
