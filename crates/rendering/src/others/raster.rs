use std::{ptr::null, rc::Rc};
use crate::{mesh::{SubMesh, vao::standard::StandardAttributeSet}, object::ToGlName, canvas::Canvas, shader::Shader, buffer::ElementBuffer, context::Context};

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
    fn vao(&self) -> &StandardAttributeSet;

    // Get the EBO handle of the object
    fn ebo(&self) -> &ElementBuffer<u32>;
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
    // Bind the shader and raster mode to the OpenGL context, and return the raw primitive type
    fn prepare(&mut self, mode: RasterMode) -> u32 {
        // Bind shader first
        unsafe {
            gl::UseProgram(self.shader.as_ref().name())
        }

        // Get the primitive type for rasterization
        let primitive = match &mode {
            RasterMode::Triangles { .. } => gl::TRIANGLES,
            RasterMode::Points { .. } => gl::POINTS,
        };

        // Set the proper drawing settings
        unsafe { self.context.set_raster_mode(mode); }
        primitive
    }

    // Rasterize the raw VAO an EBO without setting the mode or binding the shader
    unsafe fn draw_from_raw_parts(&mut self, primitive: u32, vao: u32, ebo: u32, count: u32) {
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::DrawElements(primitive, count as i32, gl::UNSIGNED_INT, null());
    }

    // Draw a single VAO and a EBO using their raw OpenGL names directly
    pub unsafe fn draw_unchecked(&mut self, vao: u32, ebo: u32, count: u32, mode: RasterMode) {
        // Bind the shader/raster modes to the context
        let primitive = self.prepare(mode);

        // Draw the VAO and EBO
        self.draw_from_raw_parts(primitive, vao, ebo, count);
    }

    // This will draw a set of attributes and indices directly onto the screen
    pub fn draw_batch<T: ToRasterBuffers>(&mut self, objects: &[&T], mode: RasterMode) {
        // Bind the shader/raster modes to the context
        let primitive = self.prepare(mode);

        // Iterate through each object and draw it
        for object in objects {
            // Get the raw OpenGL names
            let vao = object.vao();
            let ebo = object.ebo();

            unsafe {
                self.draw_from_raw_parts(primitive, vao.name(), ebo.name(), ebo.len() as u32)
            }
        }
    }
}