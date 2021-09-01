use hypo_math as math;
use hypo_others::CacheManager;
use hypo_rendering::{Model, Shader, SubShader};
use hypo_resources::ResourceManager;
use std::{ffi::c_void, mem::size_of, ptr::null};

// Constants
pub const MAX_LINE_COUNT: i32 = 8192;
pub const DRAW_DEBUG: bool = false;
// Debug renderer functionality
#[derive(Default)]
pub struct DebugRenderer {
    pub debug_primitives: Vec<DebugRendererType>,
    pub shader_name: String,
    pub vao: u32,
    pub vertices: Vec<veclib::Vector3<f32>>,
    pub colors: Vec<veclib::Vector3<f32>>,
    pub vertex_buffer: u32,
    pub colors_buffer: u32,
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

            // Generate the colors array
            gl::GenBuffers(1, &mut self.colors_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.colors_buffer);
            gl::BufferData(gl::ARRAY_BUFFER, (MAX_LINE_COUNT as usize * 2 * size_of::<f32>() * 3) as isize, null(), gl::DYNAMIC_DRAW);

            // Enable the vertex attribute
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());

            // Do the same for the color attribute
            gl::EnableVertexAttribArray(1);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.colors_buffer);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, null());

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
        // Set the shader name
        self.shader_name = Shader::new(vec!["shaders\\debug.vrsh.glsl", "shaders\\debug.frsh.glsl"], resource_manager, shader_cacher).1;
    }
    // Draw the debug renderers
    pub fn draw_debug(&mut self, vp_matrix: veclib::Matrix4x4<f32>, shader_cacher_1: &CacheManager<Shader>) {
        if !DRAW_DEBUG {
            return;
        }
        // Loop each one and construct lines out of them
        let mut lines: Vec<math::shapes::Line> = Vec::new();
        self.colors.clear();
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
                    for i in 0..(12 * 2) {
                        self.colors.push(*icolor);
                    }
                }
                DebugRendererType::SPHERE(_center, _radius, icolor) => todo!(),
                DebugRendererType::LINE(line, icolor) => {
                    // Just use the line lol
                    lines.push(*line);
                    for i in 0..2 {
                        self.colors.push(*icolor);
                    }
                }
                DebugRendererType::MODEL(_model, icolor) => todo!(),
            }
        }

        // Turn all the lines into vertices
        self.vertices.clear();
        for line in lines {
            self.vertices.push(line.point);
            self.vertices.push(line.point2);
        }

        // If the vertices changed, then edit the vertex buffer and color buffer as well
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (self.vertices.len() * size_of::<f32>() * 3) as isize,
                self.vertices.as_ptr() as *const c_void,
            );
            // Set the colors attribute
            gl::BindBuffer(gl::ARRAY_BUFFER, self.colors_buffer);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (self.colors.len() * size_of::<f32>() * 3) as isize,
                self.colors.as_ptr() as *const c_void,
            );
        }

        // Set the shader
        let shader = shader_cacher_1.get_object(self.shader_name.as_str()).unwrap();
        // Since we don't have a model matrix you can set it directly
        shader.use_shader();
        shader.set_matrix_44_uniform("vp_matrix", vp_matrix);

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
    pub fn debug_default(&mut self, default_debug_renderer_type: DefaultDebugRendererType, color: veclib::Vector3<f32>) {
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
                    .collect::<Vec<veclib::Vector3<f32>>>();
                // Add the cube debug primitive
                self.debug(DebugRendererType::CUBE(new_corner, color));
            }
            DefaultDebugRendererType::AABB(aabb) => {
                // Get the corners
                let mut corners: Vec<veclib::Vector3<f32>> = Vec::new();
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
    CUBE(Vec<veclib::Vector3<f32>>, veclib::Vector3<f32>),
    SPHERE(veclib::Vector3<f32>, f32, veclib::Vector3<f32>),
    LINE(math::shapes::Line, veclib::Vector3<f32>),
    MODEL(Model, veclib::Vector3<f32>),
}

// Kind of a wrapper around DebugRendererType, since it creates one from the data that we get
pub enum DefaultDebugRendererType {
    CUBE(veclib::Vector3<f32>, veclib::Vector3<f32>),
    AABB(math::bounds::AABB),
}

// Trait
pub trait DebugRendererable {
    // Get the debug renderer from the current struct
    fn get_debug_renderer(&self) -> DebugRendererType;
}
