use std::ops::{Index, IndexMut};

#[derive(Default)]
// Mesh indices
pub struct Indices {
    pub indices: Vec<u32>,
}

impl Indices {
    // Length and is_empty
    pub fn len(&self) -> usize {
        self.indices.len()
    }
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

// Indexing
impl Index<usize> for Indices {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.indices[index]
    }
}
impl IndexMut<usize> for Indices {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.indices[index]
    }
}

// Index builder
pub struct IndexBuilder<'a> {
    pub indices: &'a mut Indices
}

impl<'a> IndexBuilder<'a> {
    // Add a single index
    pub fn push(&mut self, index: u32) {
        self.indices.indices.push(index);
    }
    // Add a triangle
    pub fn triangle(&mut self, triangle: [u32; 3]) {
        self.indices.indices.extend_from_slice(&triangle);
    }
}