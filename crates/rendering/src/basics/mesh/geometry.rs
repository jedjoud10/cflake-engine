use super::{Mesh, VertexBuilder, Vertices, Indices, IndexBuilder};

// A mesh geometry builder
// When this gets dropped, it will apply all the changes to the model 
pub struct GeometryBuilder<'a> {
    pub vertex_builder: VertexBuilder<'a>,
    pub index_builder: IndexBuilder<'a>,
}