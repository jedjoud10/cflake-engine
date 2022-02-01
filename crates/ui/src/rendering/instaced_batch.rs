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
    mem::{size_of, ManuallyDrop},
};

// A unique identifier for each instanced batch
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct InstancedBatchIdentifier {
    // Some unique values for each batch
    pub shader: ObjectID<Shader>,
    pub texture: ObjectID<Texture>,
}

// A single instanced batch that contains some instanced arrays
pub struct InstancedBatch {
    // Instanced model
    pub vao: u32,
    pub vbo: u32,
    // Arrays
    pub screen_uvs_buf: DynamicRawBuffer<veclib::Vector4<f32>>,
    pub texture_uvs_buf: DynamicRawBuffer<veclib::Vector4<f32>>,
    pub colors_buf: DynamicRawBuffer<veclib::Vector4<f32>>,
    pub depth_buf: DynamicRawBuffer<f32>,

    // Per instance settings
    pub instances: HashMap<ElementID, usize>,
    pub instance_count: usize,
}

impl InstancedBatch {
    // Create a new empty instanced batch
    pub(crate) fn new() -> Self {
        const STARTING_CAPACITY: usize = 8;

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

        // Create the instanced arrays
        let screen_uvs_buf = DynamicRawBuffer::<veclib::Vector4<f32>>::with_capacity(gl::ARRAY_BUFFER, STARTING_CAPACITY, DYNAMIC_USAGE);
        let texture_uvs_buf = DynamicRawBuffer::<veclib::Vector4<f32>>::with_capacity(gl::ARRAY_BUFFER, STARTING_CAPACITY, DYNAMIC_USAGE);
        let depth_buf = DynamicRawBuffer::<f32>::with_capacity(gl::ARRAY_BUFFER, STARTING_CAPACITY, DYNAMIC_USAGE);
        let colors_buf = DynamicRawBuffer::<veclib::Vector4<f32>>::with_capacity(gl::ARRAY_BUFFER, STARTING_CAPACITY, DYNAMIC_USAGE);

        // Link the dynamic buffers to the vertex array now
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::EnableVertexAttribArray(0);
        }

        // Instanced arrays
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, screen_uvs_buf.buffer);
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribDivisor(1, 1);

            gl::BindBuffer(gl::ARRAY_BUFFER, texture_uvs_buf.buffer);
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribDivisor(2, 1);

            gl::BindBuffer(gl::ARRAY_BUFFER, depth_buf.buffer);
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribDivisor(3, 1);

            gl::BindBuffer(gl::ARRAY_BUFFER, colors_buf.buffer);
            gl::EnableVertexAttribArray(4);
            gl::VertexAttribDivisor(4, 1);
        }
        Self {
            vao,
            vbo,
            screen_uvs_buf,
            texture_uvs_buf,
            depth_buf,
            colors_buf,
            instance_count: 0,
            instances: HashMap::new(),
        }
    }
}
