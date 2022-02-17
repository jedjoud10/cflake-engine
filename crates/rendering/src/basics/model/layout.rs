// How we lay the VBO data for the model
#[derive(Clone, Copy)]
pub enum VertexAttributeBufferLayout {
    SeparateVBOs,
    SameVBO,
    Interleaved,
}

impl Default for VertexAttributeBufferLayout {
    fn default() -> Self {
        Self::SeparateVBOs
    }
}
