use std::{collections::{HashMap, HashSet}, mem::{ManuallyDrop, size_of}, ffi::c_void};

use multimap::MultiMap;
use ordered_vec::simple::OrderedVec;
use rendering::{basics::model::{Model2D, Model2DBuffers}, advanced::raw::dynamic_buffer::DynamicRawBuffer, utils::{UsageType, AccessType, UpdateFrequency}};

use crate::Root;

// A buffer containing the instanced model that we will use for rendering
// We will always at the end of the frame as well
pub struct UIRenderingBuffer {
    // Instanced model
    pub vao: u32,
    pub vbo: u32,

    // These are the instanced array
    pub screen_uvs_buf: DynamicRawBuffer<veclib::Vector4<f32>>,
    pub mapped_uvs_buf: DynamicRawBuffer<veclib::Vector4<f32>>,
    pub depth_buf: DynamicRawBuffer<f32>,

    // Per instance settings
    pub instance_count: usize,
}

impl UIRenderingBuffer {
    // Create a new rendering buffer with a certain capacity to hold some default elements 
    // This must be called on the render thread because that is the only place where we have an OpenGL context
    pub unsafe fn new(capacity: usize) -> Self {
        // Create all the buffers that we need for rendering
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        // How we will use the arrays in the shader
        const USAGE: UsageType = UsageType {
            access: AccessType::Write,
            frequency: UpdateFrequency::Static
        } ;

        // Create the vertex buffer and fill it up 
        let vertices = ManuallyDrop::new(vec![
            veclib::vec2(-1.0, -1.0),
            veclib::vec2(-1.0, 1.0),
            veclib::vec2(1.0, -1.0),
            veclib::vec2(1.0, 1.0_f32),
        ]);
        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, (4 * size_of::<veclib::Vector2<f32>>()) as isize, vertices.as_ptr() as *const c_void, USAGE.convert());
        
        // Create the instanced arrays
        let screen_uvs_buf = DynamicRawBuffer::<veclib::Vector4<f32>>::with_capacity(gl::ARRAY_BUFFER, capacity, USAGE);
        let mapped_uvs_buf = DynamicRawBuffer::<veclib::Vector4<f32>>::with_capacity(gl::ARRAY_BUFFER, capacity, USAGE);
        let depth_buf = DynamicRawBuffer::<f32>::with_capacity(gl::ARRAY_BUFFER, capacity, USAGE);

        // Linke the dynamic buffers to the vertex array now
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::EnableVertexAttribArray(0);

        // Instanced arrays
        gl::BindBuffer(gl::ARRAY_BUFFER, screen_uvs_buf.buffer);
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribDivisor(1, 1);
        gl::BindBuffer(gl::ARRAY_BUFFER, mapped_uvs_buf.buffer);
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribDivisor(2, 1);
        gl::BindBuffer(gl::ARRAY_BUFFER, depth_buf.buffer);
        gl::EnableVertexAttribArray(3);
        gl::VertexAttribDivisor(3, 1);

        Self {
            vao,
            vbo,
            screen_uvs_buf,
            mapped_uvs_buf,
            depth_buf,
            instance_count: 0,
        }
    }
    // Update our instanced buffers
    pub fn update_data(&mut self, root: &Root) {
        
    }
    // Draw all the elements that are part of the root
    pub fn draw(&self, root: &Root) {
    }
}