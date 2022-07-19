use crate::buffer::ElementBuffer;

// Immutable access to the mesh indices
pub struct IndicesRef<'a> {
    pub(super) buffer: &'a ElementBuffer<u32>,
}

impl IndicesRef<'_> {
    // Get an immutable reference to the inner buffer
    pub fn data(&self) -> &ElementBuffer<u32> {
        todo!()
    }

    // Get the number of indices that we have (triangles * 3)
    pub fn len(&self) -> usize {
        todo!()
    }

    // Check if the indices are valid (multiple of 3)
    pub fn is_valid(&self) -> bool {
        self.len() % 3 == 0
    }
}

// Mutable access to the mesh indices
pub struct IndicesMut<'a> {
    pub(super) vao: u32,
    pub(super) buffer: &'a mut ElementBuffer<u32>,
}

impl IndicesMut<'_> {
    // Get an immutable reference to the inner buffer
    pub fn data(&self) -> &ElementBuffer<u32> {
        todo!()
    }

    // Get a mutable reference to the inner buffer
    pub fn data_mut(&mut self) -> &mut ElementBuffer<u32> {
        todo!()
    }

    // Get the number of indices that we have
    pub fn len(&self) -> usize {
        todo!()
    }

    // Add some new triangles into the buffer
    pub fn push(&mut self, trianlge: (u32, u32, u32)) {
        todo!()
    }

    // Add multiple triangles into the buffer
    pub fn extend_from_slice(&mut self, triangles: &[(u32, u32, u32)]) {
        todo!()
    }
}

impl Drop for IndicesMut<'_> {
    fn drop(&mut self) {
        todo!()
    }
}
