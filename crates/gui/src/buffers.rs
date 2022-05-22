use std::{ffi::c_void, mem::size_of, ptr::null};

use rendering::buffer::{ArrayBuffer, BufferMode, ElementBuffer};
use rendering::context::Context;
use rendering::gl;
use rendering::gl::types::GLuint;
use rendering::object::ToGlName;

// Some pre allocated buffers that we can edit everytime we draw a specific clipped mesh
pub(crate) struct Buffers {
    // We create a single VAO and update the buffers every time we render, sounds pretty inefficient but it works
    pub(crate) vao: GLuint,

    // Very naive dynamic buffers
    pub(crate) indices: ElementBuffer,
    pub(crate) vertices: ArrayBuffer<egui::epaint::Vertex>,
}

impl Buffers {
    // Create the raw OpenGL buffers that we shall use for rendering
    pub fn new(ctx: &mut Context) -> Self {
        unsafe {
            // Create a simple VAO
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            // Dynamic raw buffers
            let indices = ElementBuffer::new(ctx, BufferMode::Resizable, &[]).unwrap();
            let vertices = ArrayBuffer::new(ctx, BufferMode::Resizable, &[]).unwrap();

            // Bind the vertex attributes
            unsafe {
                const STRIDE: i32 = size_of::<egui::epaint::Vertex>() as i32;
                gl::BindBuffer(gl::ARRAY_BUFFER, vertices.name().get());
                gl::EnableVertexAttribArray(0);
                gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, STRIDE, null());
                gl::EnableVertexAttribArray(1);
                gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, STRIDE, (size_of::<f32>() * 2) as isize as *const c_void);
                gl::EnableVertexAttribArray(2);
                gl::VertexAttribPointer(2, 4, gl::UNSIGNED_BYTE, gl::FALSE, STRIDE, (size_of::<f32>() * 4) as isize as *const c_void);

                // Unbind
                gl::BindVertexArray(0);
            }

            // Self
            println!("GUI Painter Buffers Init Successful!");
            Self { vao, indices, vertices }
        }
    }
    // Fill the buffers with new mesh data
    pub fn fill_buffers(&mut self, vertices: Vec<egui::epaint::Vertex>, indices: Vec<u32>) {
        //self.vertices.write(&vertices);
        //self.indices.write(&indices);
    }

    // Draw the buffers onto the screen
    pub fn draw(&mut self) {
        unsafe {
            //gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.indices.storage().buffer());
            //gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, null());
        }
    }
}
