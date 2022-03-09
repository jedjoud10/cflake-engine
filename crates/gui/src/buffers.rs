use std::{ffi::c_void, mem::size_of, ptr::null};

use gl::types::GLuint;
use rendering::{
    advanced::raw::{dynamic::DynamicBuffer, Buffer},
    pipeline::Pipeline,
    utils::{AccessType::ClientToServer, UpdateFrequency::Stream, UsageType},
};

// Some pre allocated buffers that we can edit everytime we draw a specific clipped mesh
pub(crate) struct Buffers {
    // We create a single VAO and update the buffers every time we render, sounds pretty inefficient but it works
    pub(crate) vao: GLuint,

    // No optimizations yet
    pub(crate) indices: DynamicBuffer<u32>,
    pub(crate) vertices: DynamicBuffer<egui::epaint::Vertex>,
}

impl Buffers {
    // Create some new buffers
    // This is guaranteed to be executed on the pipeline, so there is nothing to be worried about
    pub fn new(pipeline: &Pipeline) -> Self {
        // Create a simple VAO
        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
        }

        // Also generate the buffers
        const USAGE_TYPE: UsageType = UsageType {
            access: ClientToServer,
            frequency: Stream,
        };
        // Dynamic raw buffers
        let indices = DynamicBuffer::<u32>::new(Vec::default(), gl::ELEMENT_ARRAY_BUFFER, USAGE_TYPE, pipeline);
        let vertices = DynamicBuffer::<egui::epaint::Vertex>::new(Vec::default(), gl::ARRAY_BUFFER, USAGE_TYPE, pipeline);

        // Bind the vertex attributes
        unsafe {
            const STRIDE: i32 = size_of::<egui::epaint::Vertex>() as i32;
            gl::BindBuffer(gl::ARRAY_BUFFER, vertices.storage().buffer());
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
    // Fill the buffers with new mesh data
    pub fn fill_buffers(&mut self, vertices: Vec<egui::epaint::Vertex>, indices: Vec<u32>) {
        self.vertices.set_inner(vertices);
        self.indices.set_inner(indices);
    }
    // And draw
    pub fn draw(&mut self) {
        unsafe {
            // Le drawing
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.indices.storage().buffer());
            gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, null());
        }
    }
}
