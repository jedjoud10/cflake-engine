use std::{ffi::c_void, ptr::null};

use crate::engine::{math, rendering::model::Model};

// Debug functionality
#[derive(Default)]
pub struct Debug {
    pub debug_renderers: Vec<DebugRendererType>, 
    pub vao: u32,
}

impl Debug {
    // Generate the vao
    pub fn generate_gpu_data(&mut self) {
        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);
            gl::BufferData(gl::ARRAY_BUFFER, 1024, null(), gl::DYNAMIC_DRAW);
        }
    }
    // Draw the debug renderers
    pub fn draw_debug(&self, vp_matrix: glam::Mat4) {
        // Loop each one and construct lines out of them
        let mut lines: Vec<math::shapes::Line> = Vec::new();        
        for renderer in self.debug_renderers.iter() {
            match renderer {
                DebugRendererType::CUBE(corners) => {
                    // Turn the corners into lines
                    // Bottom
                    lines.push(math::shapes::Line::construct(corners[0], corners[1]));
                    lines.push(math::shapes::Line::construct(corners[1], corners[2]));
                    lines.push(math::shapes::Line::construct(corners[2], corners[3]));
                    lines.push(math::shapes::Line::construct(corners[3], corners[0]));

                    // Side
                    lines.push(math::shapes::Line::construct(corners[0], corners[4]));
                    lines.push(math::shapes::Line::construct(corners[1], corners[5]));
                    lines.push(math::shapes::Line::construct(corners[2], corners[6]));
                    lines.push(math::shapes::Line::construct(corners[3], corners[7]));

                    // Top
                    lines.push(math::shapes::Line::construct(corners[4], corners[5]));
                    lines.push(math::shapes::Line::construct(corners[5], corners[6]));
                    lines.push(math::shapes::Line::construct(corners[6], corners[7]));
                    lines.push(math::shapes::Line::construct(corners[7], corners[4]));
                },
                DebugRendererType::SPHERE(center, radius) => todo!(),
                DebugRendererType::LINE(line) => {
                    // Just use the line lol
                    lines.push(*line);
                },
                DebugRendererType::MODEL(model) => todo!(),
            }
        }
    
        // Turn all the lines into vertices
        let mut vertices: Vec<glam::Vec3> = Vec::new();
        for line in lines {
            vertices.push(line.point);
            vertices.push(line.point2);
        }

        // Then edit the VAO
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, vertices.len() as isize, vertices.as_ptr() as *const c_void);
        }

        // Draw each line
        unsafe {
            // Remove depth testing when rendering the debug primitives
            gl::Disable(gl::DEPTH_TEST);
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vao);
            gl::DrawArrays(gl::LINES, 0, vertices.len() as i32);
            gl::Enable(gl::DEPTH_TEST);
        }
    }
}

// The types of debug renderers
pub enum DebugRendererType {
    CUBE(Vec<glam::Vec3>),
    SPHERE(glam::Vec3, f32),
    LINE(math::shapes::Line),
    MODEL(Model),
}

// Trait
pub trait DebugRenderer {
    // Get the debug renderer from the current struct
    fn get_debug_renderer(&self) -> DebugRendererType;
}