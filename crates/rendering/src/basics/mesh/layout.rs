// How we lay the VBO data for the mesh
#[derive(Clone, Copy)]
pub enum VertexAttributeBufferLayout {
    SeparateVBOs,
    Interleaved,
}

impl Default for VertexAttributeBufferLayout {
    fn default() -> Self {
        Self::SeparateVBOs
    }
}
