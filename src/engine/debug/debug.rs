use std::{ffi::c_void, ptr::null};

use crate::engine::{core::{cacher::CacheManager, ecs::system_data::SystemEventData}, math, rendering::{model::Model, shader::{Shader, SubShader}}, resources::ResourceManager};

// Debug renderer functionality
#[derive(Default)]
pub struct DebugRenderer {
    pub debug_renderers: Vec<DebugRendererType>, 
    pub shader_name: String,
    pub vao: u32,
}

impl DebugRenderer {
    // Generate the vao and load the shader
    pub fn setup_debug_renderer(&mut self, resource_manager: &mut ResourceManager, shader_cacher: &mut (CacheManager<SubShader>, CacheManager<Shader>)) {
        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);
            gl::BufferData(gl::ARRAY_BUFFER, 1024, null(), gl::DYNAMIC_DRAW);
        }

        // Set the shader name
        self.shader_name = Shader::new(vec!["shaders\\debug.vrsh.glsl", "shaders\\debug.frsh.glsl"], resource_manager, shader_cacher).1;
    }
    // Draw the debug renderers
    pub fn draw_debug(&self, vp_matrix: glam::Mat4, data: &SystemEventData) {
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

        // Set the shader
        let shader = data.shader_cacher.1.get_object(self.shader_name.as_str()).unwrap();
        // Since we don't have a model matrix you can set it directly
        shader.set_matrix_44_uniform("mvp_matrix", vp_matrix);
        shader.set_scalar_3_uniform("debug_color", (1.0, 1.0, 1.0));

        // Draw each line
        unsafe {
            // Remove depth testing when rendering the debug primitives
            gl::Disable(gl::DEPTH_TEST);
            gl::PolygonMode(gl::FRONT, gl::LINE);
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vao);
            gl::DrawArrays(gl::LINES, 0, vertices.len() as i32);
            gl::Enable(gl::DEPTH_TEST);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
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
pub trait DebugRendererable {
    // Get the debug renderer from the current struct
    fn get_debug_renderer(&self) -> DebugRendererType;
}