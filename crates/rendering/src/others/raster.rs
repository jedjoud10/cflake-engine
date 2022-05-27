use std::ptr::null;
use crate::{mesh::SubMesh, object::ToGlName, canvas::Canvas, shader::Shader, buffer::ElementBuffer};

// Primitive types for rasterization
pub enum Primitive {
    Points, Triangles, 
}

// This trait will draw the current object onto a rasterizer using a unique shared active shader
pub trait Rasterizable {
    // Get the VAO and the index buffer for this rasterizable object
    fn buffers(&self) -> (&VertexArrayObject, &ElementBuffer);

    // Get the primitive type that we will use
    fn primitive() -> Primitive;
}

// A rasterizer is what will draw our vertices and triangles onto the screen, so we can actually see them as lit pixels
// Each rasterizer will use a unique shared shader
pub struct Rasterizer<'canvas, 'shader> {
    // The canvas we will be rasterizing onto
    pub(super) canvas: &'canvas mut Canvas,

    // The unique shader that we are using to rasterize our primitives
    pub(super) shader: &'shader mut Shader,
}

impl<'canvas, 'shader> Rasterizer<'canvas, 'shader> {
    // Draw multiple raster primitives onto the screen
    #[inline(always)]
    pub fn draw<R: Rasterizable>(&mut self, objects: &[&R]) {
        // Bind the raw shader program 
        unsafe {
           gl::UseProgram(self.shader.as_ref().name());
        }


        // Then we can rasterize the objects onto the canvas
        for object in objects {
            // Create some raster info for the current object

            unsafe {
                object.raster(self.canvas, self.shader);
            }
        }
    }
}