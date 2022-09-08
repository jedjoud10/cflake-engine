use crate::{buffer::TriangleBuffer, context::ToGlName};

// Immutable access to the mesh indices
pub struct TrianglesRef<'a> {
    pub(super) buffer: &'a TriangleBuffer<u32>,
}

impl TrianglesRef<'_> {
    // Get an immutable reference to the inner buffer
    pub fn data(&self) -> &TriangleBuffer<u32> {
        self.buffer
    }

    // Get the number of triangles that we have
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

// Mutable access to the mesh indices
pub struct TrianglesMut<'a> {
    pub(super) vao: u32,
    pub(super) buffer: &'a mut TriangleBuffer<u32>,
    pub(super) maybe_reassigned: bool,
}

impl TrianglesMut<'_> {
    // Get an immutable reference to the inner buffer
    pub fn data(&self) -> &TriangleBuffer<u32> {
        self.buffer
    }

    // Get a mutable reference to the inner buffer
    pub fn data_mut(&mut self) -> &mut TriangleBuffer<u32> {
        self.buffer
    }

    // Get the number of triangles that we have
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    // Add some new triangles into the buffer
    pub fn push(&mut self, triangle: [u32; 3]) {
        self.buffer.extend_from_slice(&[triangle])
    }

    // Add multiple triangles into the buffer
    pub fn extend_from_slice(&mut self, triangles: &[[u32; 3]]) {
        self.buffer.extend_from_slice(triangles);
    }

    // Re-bind the triangle buffer to the VAO
    // This is done automatically when "self" is dropped
    pub fn rebind(&mut self, force: bool) {
        if self.maybe_reassigned || force {
            unsafe {
                gl::VertexArrayElementBuffer(self.vao, self.buffer.name());
            }
        }

        self.maybe_reassigned = false;
    }
}

impl Drop for TrianglesMut<'_> {
    fn drop(&mut self) {
        self.rebind(false);
    }
}
