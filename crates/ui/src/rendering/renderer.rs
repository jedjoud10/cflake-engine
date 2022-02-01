use crate::RenderingBuffer;

// The renderer that will render each element using instancing
pub struct Renderer {
    // We contain the OpenGL buffer data
    buffer: RenderingBuffer,
}

impl Renderer {
    // Create a new UI renderer
    pub fn new() -> Self {
        Self {
            buffer: unsafe { RenderingBuffer::new(100) },
        }
    }
}
