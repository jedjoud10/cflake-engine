use graphics::TriangleBuffer;

use crate::RenderPath;

// Immutable access to the mesh indices
pub struct TrianglesRef<'a, R: RenderPath>(
    pub(crate) &'a R::TriangleBuffer<u32>,
);

impl<'a, R: RenderPath> TrianglesRef<'a, R> {
    // Get an immutable reference to the inner buffer
    pub fn buffer(&self) -> &'a R::TriangleBuffer<u32> {
        self.0
    }
}

// Mutable access to the mesh indices
pub struct TrianglesMut<'a, R: RenderPath>(
    pub(crate) &'a mut R::TriangleBuffer<u32>,
);

impl<'a, R: RenderPath> TrianglesMut<'a, R> {
    // Get an immutable reference to the inner buffer
    pub fn buffer(&self) -> &R::TriangleBuffer<u32> {
        self.0
    }

    // Get a mutable reference to the inner buffer
    pub fn buffer_mut(&mut self) -> &mut R::TriangleBuffer<u32> {
        self.0
    }
}
