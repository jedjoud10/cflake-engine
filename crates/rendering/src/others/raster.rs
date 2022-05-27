
// This trait will draw the current object onto a rasterizer without checking for anything
pub trait ToRasterPrimitives {
    // Draw the object onto the current rasterizer
    unsafe fn raster(&self, rasterizer: &mut Rasterizer);
}

impl ToRasterPrimitives for SubMesh {
    unsafe fn raster(&self, rasterizer: &mut Rasterizer) {
        gl::BindVertexArray(self.name());
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.indices().name());
        gl::DrawElements(gl::TRIANGLES, self.indices().len() as i32, gl::UNSIGNED_INT, null());
    }
}

// A rasterizer is what we shall use to draw some unique objects onto the canvas using a unique shader
pub struct Rasterizer<'a>(&'a mut Framebuffer);

impl<'a> Rasterizer<'a> {
}