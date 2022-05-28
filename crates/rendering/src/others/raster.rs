use std::ptr::null;
use crate::{mesh::{SubMesh, attributes::AttributeSet}, object::ToGlName, canvas::Canvas, shader::Shader, buffer::ElementBuffer};

// An object that can be rasterized and drawn onto the screen
pub trait ToRasterBuffers {
    // Get the VAO handle of the object
    fn vao(&self) -> &AttributeSet;

    // Get the EBO handle of the object
    fn ebo(&self) -> &ElementBuffer;
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
    // This will bind the underlying shader that we use to rasterize objects
    fn bind(&self) {
        unsafe {
            gl::UseProgram(self.shader.as_ref().name())
        }
    }

    // This will draw a set of attributes and indices directly onto the screen
    #[inline(always)]
    pub fn draw<T: ToRasterBuffers>(&mut self, objects: &[&T]) {
        // Bind the shader
        self.bind();
        
        // Iterate through each object and draw it
        for object in objects {
            // Get the raw OpenGL names
            let vao = object.vao();
            let ebo = object.ebo();

            unsafe {
                // Assign the values and render
                gl::BindVertexArray(vao.name());
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo.name());
                gl::DrawElements(gl::TRIANGLES, ebo.len() as i32, gl::UNSIGNED_INT, null());
            }
        }

    }
}