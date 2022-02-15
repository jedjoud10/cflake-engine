use gl::types::GLuint;
use rendering::{
    advanced::raw::dynamic_buffer::DynamicRawBuffer,
    utils::{AccessType::Read, UpdateFrequency::Stream, UsageType},
};

// Some pre allocated buffers that we can edit everytime we draw a specific clipped mesh
pub(crate) struct Buffers {
    // We create a single VAO and update the buffers every time we render, sounds pretty inefficient but it works
    vao: GLuint,

    // No optimizations yet
    indices: DynamicRawBuffer<u32>,
    positions: DynamicRawBuffer<veclib::Vector2<f32>>,
    uvs: DynamicRawBuffer<veclib::Vector2<f32>>,
}

impl Buffers {
    // Create some new buffers
    // This is guaranteed to be executed on the pipeline, so there is nothing to be worried about
    pub fn new() -> Self {
        // Create a simple VAO
        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
        }

        // Also generate the buffers
        const USAGE_TYPE: UsageType = UsageType { access: Read, frequency: Stream };
        // Dynamic raw buffers
        let indices = DynamicRawBuffer::<u32>::new(gl::ELEMENT_ARRAY_BUFFER, USAGE_TYPE);
        let positions = DynamicRawBuffer::<veclib::Vector2<f32>>::new(gl::ARRAY_BUFFER, USAGE_TYPE);
        let uvs = DynamicRawBuffer::<veclib::Vector2<f32>>::new(gl::ARRAY_BUFFER, USAGE_TYPE);

        // Self
        println!("GUI Painter Buffers Init Successful!");
        Self {
            vao: vao,
            indices,
            positions,
            uvs,
        }
    }
}

impl Default for Buffers {
    fn default() -> Self {
        Self::new()
    }
}
