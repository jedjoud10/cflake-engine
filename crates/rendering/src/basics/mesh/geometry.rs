use super::{Mesh, VertexBuilder, Vertices, Indices, IndexBuilder};

// A mesh geometry builder
// When this gets dropped, it will apply all the changes to the model 
pub struct GeometryBuilder<'a> {
    pub(crate) vertices: &'a mut Vertices,
    pub(crate) indices: &'a mut Indices
}

impl<'a> GeometryBuilder<'a> {
    // Create a new vertex builder
    pub fn vertex_builder<'b>(&'b mut self) -> VertexBuilder<'b> {
        VertexBuilder {
            vertices: self.vertices,
        }
    }
    // Create a new index builder
    pub fn index_builder<'b>(&'b mut self) -> IndexBuilder<'b> {
        IndexBuilder {
            indices: self.indices
        }
    }
}