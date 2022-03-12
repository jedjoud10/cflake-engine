use super::{IndexBuilder, Mesh, VertexBuilder};

// Mesh geometry builder
#[derive(Default)]
pub struct GeometryBuilder {
    pub vertices: VertexBuilder,
    pub indices: IndexBuilder,
}

impl GeometryBuilder {
    // Build a mesh out of a geometry builder
    pub fn build(self) -> Mesh {
        Mesh::new(self.vertices.vertices, self.indices.indices)
    }
}
