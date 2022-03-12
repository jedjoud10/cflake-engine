// Mesh indices
pub type Indices = Vec<u32>;
// Index builder
#[derive(Default)]
pub struct IndexBuilder {
    pub indices: Vec<u32>,
}

impl IndexBuilder {
    // Add a single index
    pub fn push(&mut self, index: u32) {
        self.indices.push(index);
    }
    // Add a triangle
    pub fn triangle(&mut self, triangle: [u32; 3]) {
        self.indices.extend_from_slice(&triangle);
    }
}
