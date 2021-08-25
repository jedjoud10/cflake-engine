use std::{ffi::c_void, mem::size_of, ptr::null};

use crate::engine::{
    core::cacher::CacheManager,
    math,
    rendering::{
        model::Model,
        shader::{Shader, SubShader},
    },
    resources::ResourceManager,
};

// Constants
pub const MAX_LINE_COUNT: i32 = 8192;
pub const DRAW_DEBUG: bool = true;
// Debug renderer functionality
#[derive(Default)]
pub struct DebugRenderer {
    pub debug_primitives: Vec<DebugRendererType>,
    pub shader_name: String,
    pub vao: u32,
    pub vertices: Vec<glam::Vec3>,
    pub vertex_buffer: u32,
}

impl DebugRenderer {
    // Generate the vao and load the shader
    pub fn setup_debug_renderer(&mut self, resource_manager: &mut ResourceManager, shader_cacher: &mut (CacheManager<SubShader>, CacheManager<Shader>)) {
        if !DRAW_DEBUG {
            return;
        }
        unsafe {
            // Generate the VAO
            gl::GenVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);

            // Generate the vertex array
            gl::GenBuffers(1, &mut self.vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);
            gl::BufferData(gl::ARRAY_BUFFER, (MAX_LINE_COUNT as usize * 2 * size_of::<f32>() * 3) as isize, null(), gl::DYNAMIC_DRAW);

            // Enable the attribute
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
        // Set the shader name
        self.shader_name = Shader::new(vec!["shaders\\debug.vrsh.glsl", "shaders\\debug.frsh.glsl"], resource_manager, shader_cacher).1;
    }
    // Draw the debug renderers
    pub fn draw_debug(&mut self, vp_matrix: glam::Mat4, shader_cacher_1: &CacheManager<Shader>) {
        if !DRAW_DEBUG {
            return;
        }
        // Save the color of the debugged lines
        let mut color: glam::Vec3 = glam::Vec3::ZERO;
        // Loop each one and construct lines out of them
        let mut lines: Vec<math::shapes::Line> = Vec::new();
        for renderer in self.debug_primitives.iter() {
            match renderer {
                DebugRendererType::CUBE(corners, icolor) => {
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
                    color = *icolor;
                }
                DebugRendererType::SPHERE(_center, _radius, icolor) => todo!(),
                DebugRendererType::LINE(line, icolor) => {
                    // Just use the line lol
                    lines.push(*line);
                    color = *icolor;
                }
                DebugRendererType::MODEL(_model, icolor) => todo!(),
            }
        }

        // Turn all the lines into vertices
        let mut new_vertices: Vec<glam::Vec3> = Vec::new();
        for line in lines {
            new_vertices.push(line.point);
            new_vertices.push(line.point2);
        }
        if new_vertices != self.vertices {
            self.vertices.clear();
            self.vertices.append(&mut new_vertices);
            // If the vertices changed, then edit the vertex buffer
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (self.vertices.len() * size_of::<f32>() * 3) as isize,
                    self.vertices.as_ptr() as *const c_void,
                );
            }
        }

        // Set the shader
        let shader = shader_cacher_1.get_object(self.shader_name.as_str()).unwrap();
        // Since we don't have a model matrix you can set it directly
        shader.use_shader();
        shader.set_matrix_44_uniform("vp_matrix", vp_matrix * glam::Mat4::IDENTITY);
        shader.set_scalar_3_uniform("debug_color", (color.x, color.y, color.z));

        // Draw each line
        unsafe {
            // Remove depth testing when rendering the debug primitives
            gl::Disable(gl::DEPTH_TEST);
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::LINES, 0, self.vertices.len() as i32);
            gl::Enable(gl::DEPTH_TEST);
        }
        // Clear the debug primitives we already rendered
        self.debug_primitives.clear();
    }
    // Add a debug primitive to the queue and then render it
    pub fn debug(&mut self, debug_renderer_type: DebugRendererType) {
        if !DRAW_DEBUG {
            return;
        }
        self.debug_primitives.push(debug_renderer_type);
    }
    // Add a default debug primite to the queue
    pub fn debug_default(&mut self, default_debug_renderer_type: DefaultDebugRendererType, color: glam::Vec3) {
        if !DRAW_DEBUG {
            return;
        }
        match default_debug_renderer_type {
            DefaultDebugRendererType::CUBE(center, size) => {
                // Apply the center and size
                let new_corner = math::shapes::CUBE_CORNERS
                    .to_vec()
                    .iter()
                    .map(|&x| center + (x * size) - size / 2.0)
                    .collect::<Vec<glam::Vec3>>();
                // Add the cube debug primitive
                self.debug(DebugRendererType::CUBE(new_corner, color));
            }
            DefaultDebugRendererType::AABB(aabb) => {
                // Get the corners
                let mut corners: Vec<glam::Vec3> = Vec::new();
                for corner_index in 0..8 {
                    // Get the corners from the AABB at the specified index
                    corners.push(aabb.get_corner(corner_index));
                }
                // Add the cube debug primitive
                self.debug(DebugRendererType::CUBE(corners, color));
            }
        }
    }
}

// The types of debug renderers
pub enum DebugRendererType {
    CUBE(Vec<glam::Vec3>, glam::Vec3),
    SPHERE(glam::Vec3, f32, glam::Vec3),
    LINE(math::shapes::Line, glam::Vec3),
    MODEL(Model, glam::Vec3),
}

// Kind of a wrapper around DebugRendererType, since it creates one from the data that we get
pub enum DefaultDebugRendererType {
    CUBE(glam::Vec3, glam::Vec3),
    AABB(math::bounds::AABB),
}

// Trait
pub trait DebugRendererable {
    // Get the debug renderer from the current struct
    fn get_debug_renderer(&self) -> DebugRendererType;
}
