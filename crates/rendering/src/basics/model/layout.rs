// How we lay the VBO data for the model
#[derive(Clone, Copy)]
pub enum VertexAttributeBufferLayout {
    Separate,
    Interleaved,
}

impl Default for VertexAttributeBufferLayout {
    fn default() -> Self {
        Self::Separate
    }
}