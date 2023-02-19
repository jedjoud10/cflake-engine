use graphics::TriangleBuffer;

// Immutable access to the mesh indices
pub struct TrianglesRef<'a>(pub(crate) &'a TriangleBuffer<u32>);

impl<'a> TrianglesRef<'a> {
    // Get an immutable reference to the inner buffer
    pub fn buffer(&self) -> &'a TriangleBuffer<u32> {
        self.0
    }
}

// Mutable access to the mesh indices
pub struct TrianglesMut<'a>(pub(crate) &'a mut TriangleBuffer<u32>);

impl TrianglesMut<'_> {
    // Get an immutable reference to the inner buffer
    pub fn buffer(&self) -> &TriangleBuffer<u32> {
        self.0
    }

    // Get a mutable reference to the inner buffer
    pub fn buffer_mut(&mut self) -> &mut TriangleBuffer<u32> {
        self.0
    }
}
