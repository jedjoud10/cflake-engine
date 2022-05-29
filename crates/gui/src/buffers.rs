use std::{ffi::c_void, mem::size_of, ptr::null};

use rendering::buffer::{ArrayBuffer, BufferMode, ElementBuffer};
use rendering::context::Context;
use rendering::gl;
use rendering::gl::types::GLuint;
use rendering::object::ToGlName;

// Some pre allocated buffers that we can edit everytime we draw a specific clipped mesh
pub(crate) struct Buffers {

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
                gl::BindBuffer(gl::ARRAY_BUFFER, vertices.name());
                gl::EnableVertexAttribArray(0);
                gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, STRIDE, null());
                gl::EnableVertexAttribArray(1);
                gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, STRIDE, (size_of::<f32>() * 2) as isize as *const c_void);
                gl::EnableVertexAttribArray(2);
                gl::VertexAttribPointer(2, 4, gl::UNSIGNED_BYTE, gl::FALSE, STRIDE, (size_of::<f32>() * 4) as isize as *const c_void);
            }

            // Self
            println!("GUI Painter Buffers Init Successful!");
            Self { vao, indices, vertices }
        }
    }

    // Get the vertex array buffer immutably
    // Get the vertex array buffer mutably
    // Get the element buffer immutably
    // Get the element buffer mutably
    pub fn ebo() -> &ElementBuffer<u32> {

    }

    // Fill the buffers with new mesh data
    pub fn fill_buffers(&mut self, ctx: &mut Context, vertices: Vec<egui::epaint::Vertex>, indices: Vec<u32>) {
        self.vertices.write(ctx, &vertices);
        self.indices.write(ctx, &indices);
    }
}
