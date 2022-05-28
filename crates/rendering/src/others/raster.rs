use std::{ptr::null, rc::Rc};
use crate::{mesh::{SubMesh, attributes::AttributeSet}, object::ToGlName, canvas::Canvas, shader::Shader, buffer::ElementBuffer, context::Context};

// How rasterized triangles should be culled
pub enum FaceCullMode {
    // The boolean specifies if the culling should be Counter Clockwise
    Front(bool), Back(bool),
    
    // Don't cull anything
    None,
}

// Depicts how OpenGL should draw the raster buffers
pub enum RasterMode {
    Triangles {
        cull: FaceCullMode,
    }, Points {
        diameter: f32
    }
}

// An object that can be rasterized and drawn onto the screen
pub trait ToRasterBuffers {
    // Get the VAO handle of the object
    fn vao(&self) -> &AttributeSet;

    // Get the EBO handle of the object
    fn ebo(&self) -> &ElementBuffer;
}

// A rasterizer is what will draw our vertices and triangles onto the screen, so we can actually see them as lit pixels
// Each rasterizer will use a unique shared shader
pub struct Rasterizer<'canvas, 'shader, 'context> {
    // The canvas we will be rasterizing onto
    pub(super) canvas: &'canvas mut Canvas,

    // The unique shader that we are using to rasterize our primitives
    pub(super) shader: &'shader mut Shader,

    pub(super) context: &'context mut Context,
}

impl<'canvas, 'shader, 'context> Rasterizer<'canvas, 'shader, 'context> {
    // This will bind the underlying shader that we use to rasterize objects
    fn bind(&self) {
        unsafe {
            gl::UseProgram(self.shader.as_ref().name())
        }
    }

    // This will draw a set of attributes and indices directly onto the screen
    #[inline(always)]
    pub fn draw<T: ToRasterBuffers>(&mut self, objects: &[&T], mode: RasterMode) {
        // Bind the shader
        self.bind();

        // Set the proper drawing settings
        unsafe { self.context.set_raster_mode(mode) };
        
        // Get the primitive type for rasterization
        let primitive = match mode {
            RasterMode::Triangles { .. } => gl::TRIANGLES,
            RasterMode::Points { .. } => gl::POINTS,
        };

        // Iterate through each object and draw it
        for object in objects {
            // Get the raw OpenGL names
            let vao = object.vao();
            let ebo = object.ebo();

            unsafe {
                // Assign the values and render
                gl::BindVertexArray(vao.name());
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo.name());
                gl::DrawElements(primitive, ebo.len() as i32, gl::UNSIGNED_INT, null());
            }
        }
    }
}