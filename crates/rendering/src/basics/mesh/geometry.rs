use super::{IndexBuilder, Indices, Mesh, VertexBuilder, Vertices};

// Mesh geometry builder
#[derive(Default)]
pub struct GeometryBuilder {
    pub vertices: VertexBuilder,
    pub indices: IndexBuilder,
    /*
    pub vertex_builder: VertexBuilder<'a>,
    pub index_builder: IndexBuilder<'a>,
    */
}

impl GeometryBuilder {
    // Build a mesh out of a geometry builder
    pub fn build(self) -> Mesh {
        Mesh::new(self.vertices.vertices, self.indices.indices)
    }
}
