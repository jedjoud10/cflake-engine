use super::{IndexBuilder, Indices, Mesh, VertexBuilder, Vertices};

// A mesh geometry modifier
// When this gets dropped, it will apply all the changes to the model
pub struct GeometryModifier<'a> {
    pub vertex_builder: VertexBuilder<'a>,
    pub index_builder: IndexBuilder<'a>,
}
