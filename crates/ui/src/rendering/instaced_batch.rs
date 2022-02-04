use crate::ElementID;
use rendering::{
    advanced::raw::dynamic_buffer::DynamicRawBuffer,
    basics::{shader::Shader, texture::Texture},
    object::ObjectID,
    utils::{AccessType, UpdateFrequency, UsageType},
};
use std::{
    collections::HashMap,
    ffi::c_void,
    mem::{size_of, ManuallyDrop}, ptr::null,
};

// A unique identifier for each instanced batch
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct InstancedBatchIdentifier {
    // Some unique values for each batch
    pub shader: ObjectID<Shader>,
    pub texture: ObjectID<Texture>,
}

// A single instanced batch that contains some instanced arrays
#[derive(Debug)]
pub struct InstancedBatch {
    // Instanced model
    pub vao: u32,
    pub vbo: u32,
    pub ebo: u32,
    // Arrays
    pub screen_uvs_buf: DynamicRawBuffer<veclib::Vector2<f32>>,
    pub texture_uvs_buf: DynamicRawBuffer<veclib::Vector2<f32>>,
    pub colors_buf: DynamicRawBuffer<veclib::Vector4<f32>>,
    pub depth_buf: DynamicRawBuffer<f32>,

    // Per instance settings
    pub instance_count: usize,
}

impl InstancedBatch {
    // Create a new empty instanced batch
    pub(crate) fn new() -> Self {
        const STARTING_ALLOCATED_CAPACITY_PER_INST_ATTRIBUTE: usize = 16;
        const STARTING_ALLOCATED_CAPACITY_PER_VERT_ATTRIBUTE: usize = 16;

        // Create all the buffers that we need for rendering
        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
        }

        // How we will use the arrays in the shader
        const STATIC_USAGE: UsageType = UsageType {
            access: AccessType::Write,
            frequency: UpdateFrequency::Static,
        };
        const DYNAMIC_USAGE: UsageType = UsageType {
            access: AccessType::Write,
            frequency: UpdateFrequency::Stream,
        };

        // Create the vertex buffer and fill it up
        let mut vertices = ManuallyDrop::new(vec![veclib::vec2(-1.0, -1.0), veclib::vec2(-1.0, 1.0), veclib::vec2(1.0, -1.0), veclib::vec2(1.0, 1.0_f32)]);
        let mut vbo = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (4 * size_of::<veclib::Vector2<f32>>()) as isize,
                vertices.as_ptr() as *const c_void,
                STATIC_USAGE.convert(),
            );
            ManuallyDrop::drop(&mut vertices);
        }

        // Also create the EBO
        let mut elements = ManuallyDrop::new(vec![0, 1, 2, 1, 3, 2]);
        let mut ebo = 0;
        unsafe {
            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (elements.len() * size_of::<u32>()) as isize,
                elements.as_ptr() as *const c_void,
                STATIC_USAGE.convert()
            );            
            ManuallyDrop::drop(&mut elements);
        }

        // Create the instanced arrays
        // These two will be updated on a vertex to vertex basis
        let screen_uvs_buf = DynamicRawBuffer::<veclib::Vector2<f32>>::with_capacity(gl::ARRAY_BUFFER, STARTING_ALLOCATED_CAPACITY_PER_VERT_ATTRIBUTE, DYNAMIC_USAGE);
        let texture_uvs_buf = DynamicRawBuffer::<veclib::Vector2<f32>>::with_capacity(gl::ARRAY_BUFFER, STARTING_ALLOCATED_CAPACITY_PER_VERT_ATTRIBUTE, DYNAMIC_USAGE);
        // These other two will be updated on an instance to instance basis
        let depth_buf = DynamicRawBuffer::<f32>::with_capacity(gl::ARRAY_BUFFER, STARTING_ALLOCATED_CAPACITY_PER_INST_ATTRIBUTE, DYNAMIC_USAGE);
        let colors_buf = DynamicRawBuffer::<veclib::Vector4<f32>>::with_capacity(gl::ARRAY_BUFFER, STARTING_ALLOCATED_CAPACITY_PER_INST_ATTRIBUTE, DYNAMIC_USAGE);

        // Link the static VBO buffer
        unsafe {
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, null());
        }

        // Instanced arrays
        unsafe {
            gl::EnableVertexAttribArray(1);
            gl::BindBuffer(gl::ARRAY_BUFFER, screen_uvs_buf.buffer);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 0, null());

            gl::EnableVertexAttribArray(2);
            gl::BindBuffer(gl::ARRAY_BUFFER, texture_uvs_buf.buffer);
            gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, 0, null());

            gl::EnableVertexAttribArray(3);
            gl::BindBuffer(gl::ARRAY_BUFFER, depth_buf.buffer);
            gl::VertexAttribPointer(3, 1, gl::FLOAT, gl::FALSE, 0, null());
            gl::VertexAttribDivisor(3, 1);

            gl::EnableVertexAttribArray(4);
            gl::BindBuffer(gl::ARRAY_BUFFER, colors_buf.buffer);
            gl::VertexAttribPointer(4, 4, gl::FLOAT, gl::FALSE, 0, null());
            gl::VertexAttribDivisor(4, 1);

            // Unbind
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
        Self {
            vao,
            vbo,
            ebo,
            screen_uvs_buf,
            texture_uvs_buf,
            depth_buf,
            colors_buf,
            instance_count: 0,
        }
    }
}
