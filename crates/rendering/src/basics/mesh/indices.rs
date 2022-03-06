use std::ops::{Index, IndexMut};

// Mesh indices
pub type Indices = Vec<u32>;
// Index builder
pub struct IndexBuilder<'a> {
    pub indices: &'a mut Vec<u32>,
}

impl<'a> IndexBuilder<'a> {
    // Add a single index
    pub fn push(&mut self, index: u32) {
        self.indices.push(index);
    }
    // Add a triangle
    pub fn triangle(&mut self, triangle: [u32; 3]) {
        self.indices.extend_from_slice(&triangle);
    }
}
